-- tests/fixtures/unused.lua
-- Test cases for unused variable detection.

-- VIOLATION: unused local variable
local unused_var = 42

-- CORRECT: used local variable
local used_var = 10
print(used_var)

-- CORRECT: underscore-prefixed locals should NOT be flagged
local _intentionally_unused = "ignored"

-- VIOLATION: unused function
local function unused_helper()
    return "never called"
end

-- CORRECT: used function
local function used_helper()
    return "called below"
end
print(used_helper())

-- VIOLATION: unused parameter (may be informational)
local function process(needed, _opts, unneeded)
    return needed
end
process(1, nil, 2)

-- VIOLATION: unused local in nested scope
do
    local inner_unused = "lost"
end

-- CORRECT: variable used in a different scope
local outer = 100
do
    print(outer)
end

-- VIOLATION: shadowing — inner variable shadows outer
local shadow = "outer"
do
    local shadow = "inner"
    print(shadow)
end
print(shadow)
