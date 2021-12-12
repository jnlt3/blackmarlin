# Blackmarlin

WIP UCI Chess Engine

|![](logo.jpg?raw=true "Black Marlin")|
|:--:|
|*Logo by Alex Brunetti*|


Blackmarlin is a UCI compliant chess engine fully written in Rust by Doruk Sekercioglu.

Make sure to compile the chess engine with `cargo build --release --features nnue` or run it with `cargo run --release --features nnue` in order to get the maximum strength out of the engine.

Blackmarlin doesn't come with a built-in GUI. The recommended way of playing against the engine is to get the latest release or compile it locally and use it along with a Chess GUI that supports the UCI protocol. 

The repository used for NN training is [NNUE Marlin](https://github.com/dsekercioglu/nnue_marlin)


## Black Marlin Search

- Iterative Deepening
- Aspiration Windows
- Principal Variation Search
  - Pruning
    - Reverse Futility Pruning
    - Null Move Pruning
    - Late Move Pruning
    - Futility Pruning
    - History Based Pruning
    - SEE Assisted Futility Pruning
  - Reductions
    - History Based Reductions
    - Reduce Tactical Moves Less
    - Reduce Principal Variation Nodes Less
  - Extensions
    - Singular Extensions
    - Check Extensions
  - Move Ordering
    - Move from Transposition Table
    - Winning Captures
    - Killer Heuristic
    - Null Move Threats
    - History Heuristic
    - Bad Captures
    - Under Promotions
- Lazy SMP
  - Shared Transposition Table
  - Atomic Entries

## Black Marlin Evaluation

- Hand Crafted Eval
  - Gradient Descent Based Tuning System
- NNUE
  - Skipped Connections
  - Buckets

## Black Marlin Time Management

- Search Evaluation Instability
- Failed Searches at Root
- Best Move consistency


### Iterative Deepening
Iterative Deepening is the idea of performing consecutive searches at higher depths each time.

### Aspiration Windows
Aspiration Windows are a way of reducing the search space by making the assumption that the resulting evaluation will be in `[alpha, beta]` range. If we get a score such that `alpha < score < beta`, we know that the resulting score from the search is exact and thus can be used. Otherwise, we execute another search with a larger window.

### Principal Variation Search
In Principal Variation Search (PVS), we do a search with the window `[alpha - 1, alpha]` for each move to see if it is worse than alpha efficiently. If so, we don't need to execute a full search on the given move.

### Reverse Futility Pruning
Reverse Futility Pruning (RFP) is the idea of returning the evaluation of the current position if it is above beta by a margin. This 

### Null Move Pruning
Null Move Pruning (NMP) is based on the assumption that, if we can reach beta by not making a move at lower depths, we will most likely be able to reach it anyway.

### Late Move Pruning
Late Move Pruning (LMP) is the idea that all quiet moves can be pruned after searching the first few given by the move ordering algorithm.

### Futility Pruning
Futility Pruning is the idea that quiet moves don't tend to improve positions significantly, therefore we can safely prune quiet moves in positions where the current position is evaluated to be below alpha by a margin.

### History Heuristic
History Heuristic is usually a table that keeps track of which moves failed high/low in the past. This can be used to identify moves that tend to be good in the current search space.

### History Based Pruning
History Based Pruning is the idea of using the history table to see which moves performed worse in the local search area and pruning accordingly.

### SEE Assisted Futility Pruning
Doing Futility Pruning at higher depths by correcting the evaluation based on static exchange evaluation (SEE).

