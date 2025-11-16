#!/usr/bin/env bash
# Script to create test transactions using server functions

echo "Creating test transactions..."

# Get kids
KIDS=$(curl -s http://127.0.0.1:3000/api/get_kids)
echo "Kids: $KIDS"

# Get tasks
TASKS=$(curl -s http://127.0.0.1:3000/api/get_tasks)
echo "Tasks: $TASKS"

# Extract first kid and task IDs (this is hacky but will work for testing)
KID_ID=$(echo "$KIDS" | jq -r '.[0].id')
TASK_ID=$(echo "$TASKS" | jq -r '.[0].id')

echo "Completing task $TASK_ID for kid $KID_ID"

# Complete a task
curl -X POST http://127.0.0.1:3000/api/complete_task \
  -H "Content-Type: application/json" \
  -d "{\"kid_id\":\"$KID_ID\",\"task_id\":\"$TASK_ID\"}"

echo ""
echo "Done!"
