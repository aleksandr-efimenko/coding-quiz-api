import re
import random
import requests
import uuid
import json

README_FILE = "ingest_readme.md"
BASE_URL = "http://127.0.0.1:8080"
TITLE_REGEX = r"### (.*)"

# Pattern to find questions section. It starts after "<!-- QUESTIONS_START -->"
# We will just split by "###" after checking for the marker.

def get_token():
    username = "ingest_user_" + str(uuid.uuid4())[:8]
    password = "password"
    requests.post(f"{BASE_URL}/auth/register", json={"username": username, "password": password})
    resp = requests.post(f"{BASE_URL}/auth/login", json={"username": username, "password": password})
    return resp.json().get("token")

def parse_readme():
    try:
        with open(README_FILE, "r", encoding="utf-8") as f:
            content = f.read()
    except FileNotFoundError:
        print("README not found. Please download it first.")
        return []

    # Find start
    if "<!-- QUESTIONS_START -->" in content:
        content = content.split("<!-- QUESTIONS_START -->")[1]

    # Split by ### Q. or ###
    # The format is "1. ### Question Text" or just "### Question Text"
    # We split by "###" and ignore the first empty chunk
    chunks = content.split("###")
    
    questions = []
    
    for chunk in chunks:
        chunk = chunk.strip()
        if not chunk:
            continue
            
        # First line is title
        lines = chunk.split("\n")
        title = lines[0].strip()
        
        # Rest is answer/explanation
        explanation = "\n".join(lines[1:]).strip()
        
        # Skip empty
        if not title or not explanation:
            continue
            
        questions.append({
            "title": title,
            "explanation": explanation
        })
        
    return questions

def generate_distractors(target_q, all_questions):
    # Get 3 random other questions
    others = random.sample(all_questions, 3)
    distractors = []
    for o in others:
        if o == target_q:
            continue
        # Take first sentence or ~100 chars
        # Simple heuristic: Split by dot, take first part.
        snippet = o["explanation"].split(".")[0]
        if len(snippet) > 150:
            snippet = snippet[:147] + "..."
        distractors.append(snippet)
    return distractors[:3] # Ensure max 3

def extract_correct_answer(explanation):
    # Take first sentence as correct answer summary
    snippet = explanation.split(".")[0]
    if len(snippet) > 150:
        snippet = snippet[:147] + "..."
    # Ensure it's not empty code block
    if snippet.strip().startswith("```"):
        snippet = "Refer to the code example in the explanation."
    return snippet

def ingest():
    token = get_token()
    if not token:
        print("Failed to get token")
        return

    questions_pool = parse_readme()
    print(f"Parsed {len(questions_pool)} questions.")
    
    # We want to create quizzes. Maybe 1 quiz per question is too much "Quizzes".
    # Or group them? The user said "create thousands of quizzes". 
    # Let's create 1 quiz per 5 questions? Or 1 quiz per question?
    # "Create thousands of quizzes" implies 1 quiz = 1 question or small sets.
    # Let's do 1 quiz = 1 question for simplicity unless specified otherwise,
    # or group by topic? The readme is one big list.
    # Let's do batches of 5 questions per quiz.
    
    BATCH_SIZE = 5
    
    # Get Category "JavaScript"
    cat_id = None
    cats_resp = requests.get(f"{BASE_URL}/categories")
    for c in cats_resp.json():
        if c["name"] == "JavaScript":
            cat_id = c["id"]
            break
            
    if not cat_id:
        # Create it
        c_resp = requests.post(f"{BASE_URL}/categories", json={"name": "JavaScript"}, headers={"Authorization": f"Bearer {token}"})
        cat_id = c_resp.json()["id"]

def detect_topics(title, explanation):
    text = (title + " " + explanation).lower()
    
    topics = set()
    
    # Keyword Mapping
    keywords = {
        "async": ["async", "await", "promise", "callback", "timeout", "interval"],
        "arrays": ["array", "slice", "splice", "map", "filter", "reduce", "push", "pop"],
        "objects": ["object", "prototype", "class", "this keyword", "new keyword", "constructor"],
        "functions": ["function", "arrow", "closure", "bind", "call", "apply", "iife"],
        "dom": ["dom", "element", "event", "listener", "document", "window", "browser"],    
        "types": ["typeof", "type", "null", "undefined", "nan", "coercion"],
        "storage": ["cookie", "localstorage", "sessionstorage", "indexeddb"],
        "web-workers": ["worker"],
        "security": ["xss", "csrf", "security", "cors"],
    }
    
    for topic, words in keywords.items():
        for w in words:
            if w in text:
                topics.add(topic)
                break
                
    # Always include javascript, but ensure we have at least one specific topic if possible
    final_tags = ["javascript"] + list(topics)
    
    # If no specific topic found, add 'general'
    if len(final_tags) == 1:
        final_tags.append("general")
        
    return final_tags

def ingest():
    token = get_token()
    if not token:
        print("Failed to get token")
        return

    questions_pool = parse_readme()
    print(f"Parsed {len(questions_pool)} questions.")
    
    # BATCH_SIZE = 5
    
    # Get Category "JavaScript"
    cat_id = None
    cats_resp = requests.get(f"{BASE_URL}/categories")
    for c in cats_resp.json():
        if c["name"] == "JavaScript":
            cat_id = c["id"]
            break
            
    if not cat_id:
        # Create it
        c_resp = requests.post(f"{BASE_URL}/categories", json={"name": "JavaScript"}, headers={"Authorization": f"Bearer {token}"})
        cat_id = c_resp.json()["id"]

    # Ingest 1 question per quiz to make titles more relevant? 
    # Or strict batching? Let's stick to batching but use topic detection.
    # Actually, grouping by topic would be better but hard since the pool is mixed.
    # Let's just create quizzes with mixed questions but apply the UNION of tags?
    # OR better: Create single-question quizzes? "Thousands of quizzes" might imply granular.
    # Let's stick to 5 questions per quiz. The tags for the QUIZ should be the union of tags of questions?
    # Or just "javascript" + specific ones if they are common?
    # Let's try to detect the DOMINANT topic or just list all distinctive topics found in the batch.
    
    BATCH_SIZE = 5

    for i in range(0, len(questions_pool), BATCH_SIZE):
        batch = questions_pool[i : i + BATCH_SIZE]
        
        quiz_questions = []
        batch_tags = set()
        
        for q in batch:
            correct_text = extract_correct_answer(q["explanation"])
            distractors = generate_distractors(q, questions_pool)
            
            q_tags = detect_topics(q["title"], q["explanation"])
            batch_tags.update(q_tags)
            
            options_data = [{"text": correct_text, "is_correct": True}]
            for d in distractors:
                options_data.append({"text": d, "is_correct": False})
            
            random.shuffle(options_data)
            
            quiz_questions.append({
                "text": q["title"],
                "options": options_data,
                "explanation": q["explanation"]
            })
            
        quiz_title = f"JS Interview Prep Part {i // BATCH_SIZE + 1}"
        
        # Limit tags to 5 to avoid overflow
        final_quiz_tags = list(batch_tags)
        if len(final_quiz_tags) > 5:
            final_quiz_tags = final_quiz_tags[:5]

        payload = {
            "title": quiz_title,
            "category_id": cat_id,
            "questions": quiz_questions,
            "tags": final_quiz_tags
        }
        
        print(f"Creating quiz: {quiz_title} with tags {final_quiz_tags}")
        resp = requests.post(
            f"{BASE_URL}/quizzes",
            json=payload,
            headers={"Authorization": f"Bearer {token}"}
        )
        if resp.status_code != 201:
            print(f"Failed: {resp.text}")

if __name__ == "__main__":
    ingest()
