#!/bin/bash
#
# Stress Test Script for BE-Rust-Task-Management API
# Uses ApacheBench (ab) and curl for comprehensive load testing
#

BASE_URL="http://127.0.0.1:3000"
RESULTS_DIR="/Users/a/Documents/code/be-rust-task-management/stress_test_results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

mkdir -p "$RESULTS_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}   STRESS TEST - BE-Rust-Task-Management API${NC}"
echo -e "${CYAN}   $(date)${NC}"
echo -e "${CYAN}══════════════════════════════════════════════════════════════${NC}"
echo ""

# ─── Step 1: Register a test user ─────────────────────────────────────────────
echo -e "${YELLOW}[SETUP] Registering stress test user...${NC}"
REGISTER_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/auth/register" \
    -H "Content-Type: application/json" \
    -d '{"username":"stresstest_user","email":"stresstest@example.com","password":"StressTest123!"}')
REGISTER_STATUS=$(echo "$REGISTER_RESPONSE" | tail -1)
echo -e "  Register status: $REGISTER_STATUS"

# ─── Step 2: Login to get JWT token ──────────────────────────────────────────
echo -e "${YELLOW}[SETUP] Logging in to get JWT token...${NC}"
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"email":"stresstest@example.com","password":"StressTest123!"}')
TOKEN=$(echo "$LOGIN_RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['token'])" 2>/dev/null)

if [ -z "$TOKEN" ]; then
    echo -e "${RED}[ERROR] Could not get JWT token. Login response: $LOGIN_RESPONSE${NC}"
    exit 1
fi
echo -e "  ${GREEN}Token obtained successfully${NC}"
echo ""

# ─── Step 3: Create some test tasks ──────────────────────────────────────────
echo -e "${YELLOW}[SETUP] Creating 10 seed tasks...${NC}"
TASK_IDS=()
for i in $(seq 1 10); do
    TASK_RESP=$(curl -s -X POST "$BASE_URL/api/tasks" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $TOKEN" \
        -d "{\"task_name\":\"Stress Task $i\",\"description\":\"Test task description $i\"}")
    TASK_ID=$(echo "$TASK_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])" 2>/dev/null)
    if [ -n "$TASK_ID" ]; then
        TASK_IDS+=("$TASK_ID")
    fi
done
echo -e "  ${GREEN}Created ${#TASK_IDS[@]} tasks${NC}"
echo ""

# Function to parse AB output
parse_ab_output() {
    local output="$1"
    local label="$2"
    
    local rps=$(echo "$output" | grep "Requests per second" | awk '{print $4}')
    local tpr=$(echo "$output" | grep "Time per request" | head -1 | awk '{print $4}')
    local failed=$(echo "$output" | grep "Failed requests" | awk '{print $3}')
    local non2xx=$(echo "$output" | grep "Non-2xx" | awk '{print $3}')
    local total_time=$(echo "$output" | grep "Time taken for tests" | awk '{print $5}')
    local p50=$(echo "$output" | grep "  50%" | awk '{print $2}')
    local p90=$(echo "$output" | grep "  90%" | awk '{print $2}')
    local p95=$(echo "$output" | grep "  95%" | awk '{print $2}')
    local p99=$(echo "$output" | grep "  99%" | awk '{print $2}')
    local transfer=$(echo "$output" | grep "Transfer rate" | awk '{print $3, $4}')
    
    echo "  ├─ Requests/sec:    ${rps:-N/A}"
    echo "  ├─ Time/request:    ${tpr:-N/A} ms"
    echo "  ├─ Failed requests: ${failed:-0}"
    echo "  ├─ Non-2xx:         ${non2xx:-0}"
    echo "  ├─ Total time:      ${total_time:-N/A} sec"
    echo "  ├─ Transfer rate:   ${transfer:-N/A}"
    echo "  └─ Percentiles:     p50=${p50:-N/A}ms  p90=${p90:-N/A}ms  p95=${p95:-N/A}ms  p99=${p99:-N/A}ms"
}

# ─── Test Configuration ─────────────────────────────────────────────────────
CONCURRENCY_LEVELS=(10 50 100 200)
TOTAL_REQUESTS=1000

echo -e "${CYAN}══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}   TEST CONFIGURATION${NC}"
echo -e "${CYAN}   Total requests per test: $TOTAL_REQUESTS${NC}"
echo -e "${CYAN}   Concurrency levels: ${CONCURRENCY_LEVELS[*]}${NC}"
echo -e "${CYAN}══════════════════════════════════════════════════════════════${NC}"
echo ""

# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║ TEST 1: POST /api/auth/login (Auth performance)                         ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}TEST 1: POST /api/auth/login${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

for c in "${CONCURRENCY_LEVELS[@]}"; do
    echo -e "\n${YELLOW}  ▶ Concurrency: ${c}${NC}"
    OUTPUT=$(ab -n $TOTAL_REQUESTS -c $c \
        -H "Content-Type: application/json" \
        -p <(echo '{"email":"stresstest@example.com","password":"StressTest123!"}') \
        -T "application/json" \
        "$BASE_URL/api/auth/login" 2>&1)
    echo "$OUTPUT" > "$RESULTS_DIR/login_c${c}.txt"
    parse_ab_output "$OUTPUT" "Login c=$c"
done

# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║ TEST 2: GET /api/auth/me (Read user profile)                            ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}TEST 2: GET /api/auth/me (Protected endpoint)${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

for c in "${CONCURRENCY_LEVELS[@]}"; do
    echo -e "\n${YELLOW}  ▶ Concurrency: ${c}${NC}"
    OUTPUT=$(ab -n $TOTAL_REQUESTS -c $c \
        -H "Authorization: Bearer $TOKEN" \
        "$BASE_URL/api/auth/me" 2>&1)
    echo "$OUTPUT" > "$RESULTS_DIR/get_me_c${c}.txt"
    parse_ab_output "$OUTPUT" "GetMe c=$c"
done

# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║ TEST 3: POST /api/tasks (Create task - write heavy)                     ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}TEST 3: POST /api/tasks (Create Task - Write Heavy)${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

for c in "${CONCURRENCY_LEVELS[@]}"; do
    echo -e "\n${YELLOW}  ▶ Concurrency: ${c}${NC}"
    OUTPUT=$(ab -n $TOTAL_REQUESTS -c $c \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -p <(echo '{"task_name":"Stress Created Task","description":"Created during stress test"}') \
        -T "application/json" \
        "$BASE_URL/api/tasks" 2>&1)
    echo "$OUTPUT" > "$RESULTS_DIR/create_task_c${c}.txt"
    parse_ab_output "$OUTPUT" "CreateTask c=$c"
done

# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║ TEST 4: GET /api/tasks (List all tasks - read heavy)                    ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}TEST 4: GET /api/tasks (List All Tasks - Read Heavy)${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

for c in "${CONCURRENCY_LEVELS[@]}"; do
    echo -e "\n${YELLOW}  ▶ Concurrency: ${c}${NC}"
    OUTPUT=$(ab -n $TOTAL_REQUESTS -c $c \
        -H "Authorization: Bearer $TOKEN" \
        "$BASE_URL/api/tasks" 2>&1)
    echo "$OUTPUT" > "$RESULTS_DIR/get_all_tasks_c${c}.txt"
    parse_ab_output "$OUTPUT" "GetAllTasks c=$c"
done

# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║ TEST 5: GET /api/tasks/my (User's tasks)                                ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}TEST 5: GET /api/tasks/my (User's Tasks)${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

for c in "${CONCURRENCY_LEVELS[@]}"; do
    echo -e "\n${YELLOW}  ▶ Concurrency: ${c}${NC}"
    OUTPUT=$(ab -n $TOTAL_REQUESTS -c $c \
        -H "Authorization: Bearer $TOKEN" \
        "$BASE_URL/api/tasks/my" 2>&1)
    echo "$OUTPUT" > "$RESULTS_DIR/get_my_tasks_c${c}.txt"
    parse_ab_output "$OUTPUT" "GetMyTasks c=$c"
done

# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║ TEST 6: GET /api/tasks/{id} (Single task by ID)                         ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}TEST 6: GET /api/tasks/{id} (Single Task by ID)${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ ${#TASK_IDS[@]} -gt 0 ]; then
    TEST_TASK_ID="${TASK_IDS[0]}"
    for c in "${CONCURRENCY_LEVELS[@]}"; do
        echo -e "\n${YELLOW}  ▶ Concurrency: ${c}${NC}"
        OUTPUT=$(ab -n $TOTAL_REQUESTS -c $c \
            -H "Authorization: Bearer $TOKEN" \
            "$BASE_URL/api/tasks/$TEST_TASK_ID" 2>&1)
        echo "$OUTPUT" > "$RESULTS_DIR/get_task_by_id_c${c}.txt"
        parse_ab_output "$OUTPUT" "GetTaskById c=$c"
    done
else
    echo -e "${RED}  Skipped: No task IDs available${NC}"
fi

# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║ TEST 7: GET /api/tasks/search?q=Stress (Search tasks)                   ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}TEST 7: GET /api/tasks/search?q=Stress (Search Tasks)${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

for c in "${CONCURRENCY_LEVELS[@]}"; do
    echo -e "\n${YELLOW}  ▶ Concurrency: ${c}${NC}"
    OUTPUT=$(ab -n $TOTAL_REQUESTS -c $c \
        -H "Authorization: Bearer $TOKEN" \
        "$BASE_URL/api/tasks/search?q=Stress" 2>&1)
    echo "$OUTPUT" > "$RESULTS_DIR/search_tasks_c${c}.txt"
    parse_ab_output "$OUTPUT" "SearchTasks c=$c"
done

# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║ TEST 8: PUT /api/tasks/{id} (Update task)                               ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}TEST 8: PUT /api/tasks/{id} (Update Task)${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ ${#TASK_IDS[@]} -gt 0 ]; then
    UPDATE_TASK_ID="${TASK_IDS[1]:-${TASK_IDS[0]}}"
    for c in "${CONCURRENCY_LEVELS[@]}"; do
        echo -e "\n${YELLOW}  ▶ Concurrency: ${c}${NC}"
        OUTPUT=$(ab -n $TOTAL_REQUESTS -c $c \
            -H "Authorization: Bearer $TOKEN" \
            -H "Content-Type: application/json" \
            -u PUT \
            -p <(echo '{"task_name":"Updated Stress Task","description":"Updated during stress test"}') \
            -T "application/json" \
            "$BASE_URL/api/tasks/$UPDATE_TASK_ID" 2>&1)
        echo "$OUTPUT" > "$RESULTS_DIR/update_task_c${c}.txt"
        parse_ab_output "$OUTPUT" "UpdateTask c=$c"
    done
else
    echo -e "${RED}  Skipped: No task IDs available${NC}"
fi

# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║ TEST 9: Endurance / Soak Test (sustained load)                          ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}TEST 9: Endurance / Soak Test (5000 requests, c=50)${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

echo -e "\n${YELLOW}  ▶ GET /api/tasks/my (5000 requests, c=50)${NC}"
OUTPUT=$(ab -n 5000 -c 50 \
    -H "Authorization: Bearer $TOKEN" \
    "$BASE_URL/api/tasks/my" 2>&1)
echo "$OUTPUT" > "$RESULTS_DIR/soak_test.txt"
parse_ab_output "$OUTPUT" "SoakTest"

# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║ TEST 10: Spike Test (burst of traffic)                                  ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}TEST 10: Spike Test (500 requests, c=500)${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

echo -e "\n${YELLOW}  ▶ GET /api/auth/me (500 requests, c=500)${NC}"
OUTPUT=$(ab -n 500 -c 500 \
    -H "Authorization: Bearer $TOKEN" \
    "$BASE_URL/api/auth/me" 2>&1)
echo "$OUTPUT" > "$RESULTS_DIR/spike_test.txt"
parse_ab_output "$OUTPUT" "SpikeTest"

# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║ TEST 11: Unauthorized access stress test                                ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}TEST 11: Unauthorized Access (No Token, 1000 req, c=100)${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

echo -e "\n${YELLOW}  ▶ GET /api/tasks (No auth token)${NC}"
OUTPUT=$(ab -n 1000 -c 100 \
    "$BASE_URL/api/tasks" 2>&1)
echo "$OUTPUT" > "$RESULTS_DIR/unauthorized_test.txt"
parse_ab_output "$OUTPUT" "UnauthorizedTest"

# ╔═══════════════════════════════════════════════════════════════════════════╗
# ║ TEST 12: Mixed Workload (concurrent reads + writes)                     ║
# ╚═══════════════════════════════════════════════════════════════════════════╝
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}TEST 12: Mixed Workload (Concurrent reads + writes)${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

echo -e "\n${YELLOW}  ▶ Running reads and writes simultaneously...${NC}"

# Run reads in background
ab -n 500 -c 50 \
    -H "Authorization: Bearer $TOKEN" \
    "$BASE_URL/api/tasks/my" > "$RESULTS_DIR/mixed_read.txt" 2>&1 &
READ_PID=$!

# Run writes simultaneously
ab -n 500 -c 50 \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -p <(echo '{"task_name":"Mixed Test Task","description":"Created during mixed workload test"}') \
    -T "application/json" \
    "$BASE_URL/api/tasks" > "$RESULTS_DIR/mixed_write.txt" 2>&1 &
WRITE_PID=$!

wait $READ_PID
wait $WRITE_PID

echo -e "\n  ${CYAN}[READ RESULTS]${NC}"
parse_ab_output "$(cat $RESULTS_DIR/mixed_read.txt)" "MixedRead"
echo -e "\n  ${CYAN}[WRITE RESULTS]${NC}"
parse_ab_output "$(cat $RESULTS_DIR/mixed_write.txt)" "MixedWrite"

# ─── Summary ───────────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}   STRESS TEST COMPLETE${NC}"
echo -e "${CYAN}   Results saved to: $RESULTS_DIR${NC}"
echo -e "${CYAN}   $(date)${NC}"
echo -e "${CYAN}══════════════════════════════════════════════════════════════${NC}"
