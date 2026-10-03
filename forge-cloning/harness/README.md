# ForkProbe harness

Forge commit fd5c996 (2026-09-30), built per /mnt/project-files/forge-runner/README.md. Decks: forge-runner/decks/delver.dck vs jund.dck (placeholder lists).

```
# compile (patched GameCopier first so the PRUNE_HIDDEN_INFO flag is settable)
JAR=/home/claude/card-forge/forge/forge-gui-desktop/target/forge-gui-desktop-2.0.16-SNAPSHOT-jar-with-dependencies.jar
mkdir -p classes_p classes
# apply prune-flag.patch to a copy of forge-ai/.../simulation/GameCopier.java, compile it into classes_p
javac -nowarn -d classes -cp classes_p:$JAR ForkProbe.java
# run from forge-gui/ (Forge finds ./res relative to cwd); args: deckDir deck1 deck2 seed nGames [prune]
java -Djava.awt.headless=true -Xmx2g -XX:+UseParallelGC -cp classes_p:classes:$JAR probe.ForkProbe /path/decks delver.dck jund.dck 31 6
python3 aggregate.py data/c_s31.out data/c_s32.out
```
`prune` mode flips GameCopier.PRUNE_HIDDEN_INFO on and runs the prune+sample+simulate pipeline (`prune_probe` lines). Raw output of the runs behind the report is in ../data/ (c_* baseline, p_* prune, st_* stack probes).
Nothing in the Forge checkout was modified; the patch is applied only to a copy compiled ahead of the jar on the classpath.
