import requests
import json
import uuid
import os
import glob

BASE_URL = "http://127.0.0.1:8080"
DATA_DIR = "."

def get_token():
    # Register/Login specific user for seeding
    username = "seed_user_" + str(uuid.uuid4())[:8]
    password = "seedpassword"
    
    # Register
    requests.post(f"{BASE_URL}/auth/register", json={
        "username": username,
        "password": password
    })
    
    # Login
    resp = requests.post(f"{BASE_URL}/auth/login", json={
        "username": username,
        "password": password
    })
    
    if resp.status_code == 200:
        return resp.json()["token"]
    else:
        print(f"Login failed: {resp.text}")
        return None

def get_or_create_category(name, token):
    # Normalize name (Capitalize first letter)
    name = name.capitalize()
    
    # List categories
    resp = requests.get(f"{BASE_URL}/categories")
    if resp.status_code == 200:
        for cat in resp.json():
            if cat["name"].lower() == name.lower():
                return cat["id"]
    
    # Create if not exists
    resp = requests.post(
        f"{BASE_URL}/categories",
        json={"name": name},
        headers={"Authorization": f"Bearer {token}"}
    )
    if resp.status_code == 201:
        print(f"Created category: {name}")
        return resp.json()["id"]
    return None

def seed_quiz_file(filepath, token):
    try:
        with open(filepath, 'r') as f:
            quiz_data = json.load(f)
    except Exception as e:
        print(f"Error reading {filepath}: {e}")
        return

    # Determine category from folder name if inside data/
    # e.g., data/javascript/quiz.json -> category "Javascript"
    category_id = quiz_data.get("category_id")
    
    if category_id is None:
        parts = filepath.split(os.sep)
        if len(parts) > 1 and parts[-2] != "data": # Assume parent folder is category
            cat_name = parts[-2]
            category_id = get_or_create_category(cat_name, token)
            if category_id:
                quiz_data["category_id"] = category_id
                print(f"Auto-assigned category '{cat_name}' to {quiz_data['title']}")

    print(f"Seeding quiz: {quiz_data['title']}")
    
    resp = requests.post(
        f"{BASE_URL}/quizzes",
        json=quiz_data,
        headers={"Authorization": f"Bearer {token}"}
    )
    
    if resp.status_code == 201:
        print(f"SUCCESS: Created quiz '{quiz_data['title']}'")
    else:
        print(f"FAILED: {resp.status_code} - {resp.text}")

def seed_all():
    token = get_token()
    if not token:
        return

    print("Authenticated successfully. Starting seed...")
    
    # Walk through data directory
    for root, dirs, files in os.walk(DATA_DIR):
        for file in files:
            if file.endswith(".json"):
                filepath = os.path.join(root, file)
                seed_quiz_file(filepath, token)

if __name__ == "__main__":
    seed_all()
