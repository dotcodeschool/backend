> [!CAUTION]
> **This repository has been archived and is no longer maintained.**
> 
> This Actix-web backend was the central service managing student repositories, test submissions, and test results. Dot Code School has moved to a fully static architecture — courses are ingested from gitorial repos at build time and deployed via Vercel, with no backend service.
> 
> See [dotcodeschool/frontend](https://github.com/dotcodeschool/frontend) for the current platform.

---

# Dot Code School Backend

This is the backend service for Dot Code School, a platform for interactive coding education. It manages repositories, test submissions, and test results for students working through coding courses.

## Overview

The Dot Code School backend provides a REST API that handles:
- Repository creation and management for students
- Test submissions and results tracking
- Course data retrieval
- User-repository relationships

The backend connects to a MongoDB database for data storage and uses Redis for log streaming during test runs.

## Features

- **Repository Management**: Create and manage Git repositories from templates
- **Test Execution**: Process test submissions and track results
- **Log Streaming**: Stream test logs in real-time via Redis
- **Course Management**: Retrieve course information and track student progress
- **Test Results**: Store and retrieve test results, including the latest results for each test

## Technology Stack

- **Rust**: Core programming language
- **Actix-web**: Web framework for the REST API
- **MongoDB**: Database for storing repositories, users, courses, and test logs
- **Redis**: Used for streaming test logs during test execution
- **WebSockets**: For real-time communication during test runs

## Getting Started

### Prerequisites

- Rust (latest stable version)
- MongoDB
- Redis
- Git server (for repository creation)

### Environment Variables

Create a `.env` file in the root directory with the following variables:

```
MONGODB_URI=mongodb://localhost:27017
REDIS_URI=redis://localhost:6379
WS_URL=ws://localhost:8080/ws
PORT=8080
BEARER_TOKEN_SECRET=your_secret_token  # Optional, for git server authentication
```

### Installation and Setup

1. Clone the repository
2. Install dependencies: `cargo build`
3. Run the server: `cargo run`

The server will start on the port specified in your `.env` file (default: 8080).

## API Endpoints

### Repositories

- `POST /api/v0/repository` - Create a new repository
  ```json
  {
    "repo_template": "rust-state-machine",
    "user_id": "user_object_id",
    "expected_practice_frequency": "every_day",
    "is_reminder_enabled": true
  }
  ```

- `GET /api/v0/repository/{repo_name}` - Get repository details

- `PUT /api/v0/repository/{repo_name}` - Update repository settings
  ```json
  {
    "expected_practice_frequency": "once_a_week",
    "is_reminder_enabled": true,
    "test_ok": true,
    "tests_queue": ["test1", "test2"]
  }
  ```

### Test Logs

- `POST /api/v0/test-log` - Add a new test log entry
  ```json
  {
    "test_slug": "balances-exists",
    "passed": false,
    "timestamp": "2025-05-07T01:42:21.688424Z",
    "section_name": "The Balances Pallet",
    "lesson_name": "Creating a Balances Pallet",
    "lesson_slug": "2-1",
    "test_name": "balances exists",
    "repo_name": "36f6b5be71628a93"
  }
  ```

- `GET /api/v0/test-logs/{repo_name}` - Get latest test results for a repository

### Submissions

- `POST /api/v0/submission` - Create a new submission
  ```json
  {
    "repo_name": "36f6b5be71628a93",
    "commit_sha": "abc123def456"
  }
  ```

- `PUT /api/v0/submission/{logstream_id}` - Update submission status
  ```json
  {
    "test_status": {
      "test1": "passed",
      "test2": "failed"
    }
  }
  ```

### Courses

- `GET /api/v0/course/{course_id}` - Get course details

## Testing the API

You can test the API endpoints using curl:

```bash
# Get latest test logs for a repository
curl -X GET http://localhost:8080/api/v0/test-logs/36f6b5be71628a93

# Create a new repository
curl -X POST http://localhost:8080/api/v0/repository \
  -H "Content-Type: application/json" \
  -d '{"repo_template":"rust-state-machine","user_id":"60d5ec9f1c9d440000000000","expected_practice_frequency":"every_day","is_reminder_enabled":true}'
```

## Database Structure

The backend uses the following MongoDB collections:

- **repositories**: Stores repository information including template, user relationships, and practice frequency settings
- **users**: Stores user information and relationships to repositories
- **courses**: Stores course information including lessons and sections
- **submissions**: Stores submission information including commit SHA and logstream details
- **testLogs**: Stores test log entries with results for each test run

## Client Workflow

When a user wants to test their code:

1. The user runs `dotcodeschool test` in their repository
2. The CLI sends a request to create a submission with the repository name and commit SHA
3. The backend returns a logstream URL, logstream ID, WebSocket URL, and tester URL
4. The CLI runs the tests and streams the logs to the provided logstream URL
5. Test results are stored in the database as test log entries
6. The frontend can fetch the latest test results using the `/api/v0/test-logs/{repo_name}` endpoint

## Contributing

To contribute to this project:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests to ensure everything works
5. Commit your changes following the Commitizen format:
   ```
   feat(api): add endpoint to fetch latest test logs by repository
   
   This commit adds a new API endpoint that returns only the latest test results.
   ```
6. Push to your branch
7. Open a Pull Request

### Development Guidelines

- Follow Rust best practices and code style
- Add appropriate error handling
- Write tests for new functionality
- Update documentation when adding or changing features
- Use meaningful commit messages following the Conventional Commits specification
