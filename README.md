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

### Late Move Reductions
Late Move Reductions (LMR) are a way of proving if a move is lower than alpha quickly. This is done by searching moves that are expected to underperform at a lower depth.

### History Based Reductions
History Based Reductions are the idea of adjusting LMR reductions based on the history score of a move.

### Extensions
Usually the search algortihm is optimized by pruning or reducing where moves aren't promising. However, it can also be done by executing a deeper search on promising moves.

### Check Extensions
We don't want to miss three-fold repetition lines or forcing sequences, so we extend moves that give a check.

### Singular Extensions
If a move is better than all other moves by a margin, we can extend the search on this move. Candidate singular moves are usually decided based on the transposition table.


### Move Ordering
In order to maximize the efficiency of alpha beta search, we optimally want to try the best moves first.

### Move from Transposition Table
The Transposition Table (TT) gives us the move we have found to be the best in case we have visited the position previously. TT moves are one of the most crucial ways of improving move ordering.

### Winning Captures
We can evaluate each move by using SEE and put winning captures first in move ordering.

### Killer Heuristic
Killer Heuristic is the idea of using quiet moves that produced a beta cutoff in sibling nodes. This is based on the assumption that a move doesn't change the position a lot and we can benefit from moves that worked in very similar positions.

### Null Move Threats
A Null Move Threat is the best move we get after executing a search for Null Move Pruning. This move can later be used to augment move ordering as it'll likely eliminate moves that don't defend a threat.

### History Based Quiet Ordering
We can order quiet moves based on their history scores in order to search quiets that historically performed better than others first.

### Bad Captures
Bad Captures are moves that most likely make the position worse. They are usually put last as they rarely improve the position.

### Underpromotions
There isn't really a reason to promote to a piece other than the queen in most positions. All promotions other than queen promotions are put to the very last of the move ordering list.


### Lazy SMP
Lazy SMP stands for Lazy Simulatenous Multi Processing. The idea behind Lazy SMP is to execute the search function on N many threads and share the hash table. This allows for faster filling of the hash table, resulting in a faster and wider search.

### Atomic Keys and Entries
In order to prevent data races and data corruption, we can use the "XOR Trick". Instead of saving the hash of the position directlly, we XOR it with the related entry. When we access the table later on, we can revert the hash back with the corresponding entry. This ensures a different hash will be produced in case the entry was corrupted.


### Hand Crafted Evaluation (HCE)
Prior to NNUE, most engines used a hand crafted evaluation function. HCE included hand picked features such as material, pawn structure, piece squares and similar. These values quickly get overwhelming to tune, in order to solve this problem, most engines employ a strategy called "Texel Tuning". The idea behind Texel Tuning is to optimize the evaluation function in such a way that it'll reflect the result of the game the best. Once we obtain a loss function, we can use metaheuristics such as genetic algorithms or gradient based algorithms such as AdaGrad to tune the hand crafted evaluation function.


### Neural Networks
A Machine Learned model that has information on all the board is naturally expected to perform better than hand picked features. 

#### Training the Neural Network
There are many distinct ways of training a neural network for chess however the one that is employed the most is using the evaluation score from a low depth search. This results in evaluations that see more into the future and produce neural networks more capable than HCE functions.

#### Data Generation
Data is usually generated by self-play where the engine plays against itself at a fixed depth/node count. This enables us to also keep track of win draw loss (WDL) information to be later used in training.

#### Training
In order keep the weights low and make the neural network more aware of evaluations close to 0, the evaluation score is usually scaled and passed through sigmoid. Usually using a formula along the lines of  `sigmoid(eval * SCALE)`. We can also add a WDL term to our train set in order to make the networ more aware of how winnable positions are. This is can be done by taking the weighted average of WDL and evaluation. 


### Efficiently Updatable Neural Networks (NNUE)
Neural Networks are the state of the art machine learning models. Although they have one problem: they are usually very slow in execution. NNUE solves this problem by doing incremental updates.

#### Incremental Updates
Alpha-Beta Search has one feature that allows us to efficiently update our neural networks. Since a depth first search is employed, a series of moves are played one after the other only causing slight changes in the actual input. It is possible to take advantage of this and only calculate what has changed in the network than propogate forward completely.