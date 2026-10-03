#!/bin/sh
# strips Forge's startup chatter from run output
grep -v "JAVA_TOOL\|not assigned to any set\|Upcoming set\|Read cards\|Language\|ThreadUtil"
