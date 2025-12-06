import requests
import json
import sys

BASE_URL = "http://127.0.0.1:8080"
USERNAME = "crud_user"
PASSWORD = "password123"

def main():
    # 1. Register User
    print("Registering user...")
    resp = requests.post(f"{BASE_URL}/auth/register", json={"username": USERNAME, "password": PASSWORD})
    print(f"Register status: {resp.status_code}")
    
    # 2. Login
    print("Logging in...")
    resp = requests.post(f"{BASE_URL}/auth/login", json={"username": USERNAME, "password": PASSWORD})
    if resp.status_code != 200:
        print("Login failed")
        sys.exit(1)
    token = resp.json()["token"]
    headers = {"Authorization": f"Bearer {token}"}
    
    # 3. Create Quiz
    print("Creating quiz...")
    payload = {
        "title": "Original Title",
        "category_id": None,
        "questions": [
            {
                "text": "Quest 1",
                "explanation": "Exp 1",
                "options": [{"text": "Opt 1", "is_correct": True}]
            }
        ],
        "tags": ["tag1", "tag2"]
    }
    resp = requests.post(f"{BASE_URL}/quizzes", json=payload, headers=headers)
    if resp.status_code != 201:
        print(f"Create failed: {resp.text}")
        sys.exit(1)
    quiz_id = resp.json()["id"]
    print(f"Created quiz {quiz_id}")
    
    # 4. Verify Create
    print("Verifying initial state...")
    resp = requests.get(f"{BASE_URL}/quizzes/{quiz_id}", headers=headers)
    quiz = resp.json()
    assert quiz["title"] == "Original Title"
    assert "tag1" in quiz["tags"]
    
    # 5. Update Quiz
    print("Updating quiz...")
    update_payload = {
        "title": "Updated Title",
        "tags": ["tag1", "new_tag"]
    }
    resp = requests.put(f"{BASE_URL}/quizzes/{quiz_id}", json=update_payload, headers=headers)
    if resp.status_code != 200:
        print(f"Update failed: {resp.text}")
        sys.exit(1)
    
    updated_quiz = resp.json()
    assert updated_quiz["title"] == "Updated Title"
    assert "new_tag" in updated_quiz["tags"]
    assert "tag2" not in updated_quiz["tags"]
    print("Update verified via response")
    
    # 6. Verify Update persistency
    resp = requests.get(f"{BASE_URL}/quizzes/{quiz_id}", headers=headers)
    quiz = resp.json()
    assert quiz["title"] == "Updated Title"
    print("Update persistency verified")
    
    # 7. Delete Quiz
    print("Deleting quiz...")
    resp = requests.delete(f"{BASE_URL}/quizzes/{quiz_id}", headers=headers)
    if resp.status_code != 204:
        print(f"Delete failed: {resp.status_code} {resp.text}")
        sys.exit(1)
    print("Delete successful")
    
    # 8. Verify Delete
    print("Verifying deletion...")
    resp = requests.get(f"{BASE_URL}/quizzes/{quiz_id}", headers=headers)
    if resp.status_code != 404:
        print(f"Quiz still exists! Status: {resp.status_code}")
        sys.exit(1)
    print("Deletion verified")
    print("ALL CRUD CHECKS PASSED")

if __name__ == "__main__":
    main()
