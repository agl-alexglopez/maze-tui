# Multithreading with Mazes

> **Note: This is a companion project to my original maze repository written in C++ ([multithreading-with-mazes](https://github.com/agl-alexglopez/multithreading-with-mazes/tree/main)). There are a number of benefits I have found in the rust version. First, it seems that this project can build on Linux, Mac, and Windows. The Windows version has slow performance and I'm working on that, but the project should run just fine everywhere else. This project is built and run with cargo.**

![wilson-demo](/images/wilson-demo.png)

## Quick Start Guide

This project is a command line application that can be run with various combinations of commands. The basic principle behind the commands is that you can ask for any combination of settings, include any settings, exclude any settings, and the program will just work. There are sensible defaults for every flag so experiment with different combinations and tweaks until you get what you are looking for. To start, you should focus mainly on how big you want the maze to be, what algorithm you want to generate it, what algorithm you want to solve it, and if any of these algorithms should be animated in real time. I would reccomend using cargo to build the project.

```zsh
$ cd run_maze/
$ cargo build --release
$ cargo run --bin run_maze
```

If you would rather just see some cool mazes right away, run the demo I have included. It runs infinite random permutations of maze builder and solver animations so you can see a wide range of what the project has to offer. Stop the loop at any time with `CTRL<C>`.

```zsh
$ cd run_maze/
$ cargo build --release
$ cargo run --bin demo

# Or set the rows and columns to your liking for bigger or smaller demo mazes.
$ cargo run --bin demo -- -r 50 -c 50
```
If you wish to dive into the more specific `run_maze` program, here is the help message that comes with the `-h` flag to get started.

Use flags, followed by arguments, in any order:

- `-r` Rows flag. Set rows for the maze.
	- Any number > 7. Zoom out for larger mazes!
- `-c` Columns flag. Set columns for the maze.
	- Any number > 7. Zoom out for larger mazes!
- `-b` Builder flag. Set maze building algorithm.
	- `rdfs` - Randomized Depth First Search.
	- `kruskal` - Randomized Kruskal's algorithm.
	- `prim` - Randomized Prim's algorithm.
	- `eller` - Randomized Eller's algorithm.
	- `wilson` - Loop-Erased Random Path Carver.
	- `wilson-walls` - Loop-Erased Random Wall Adder.
	- `fractal` - Randomized recursive subdivision.
	- `grid` - A random grid pattern.
	- `arena` - Open floor with no walls.
- `-m` Modification flag. Add shortcuts to the maze.
	- `cross` - Add crossroads through the center.
	- `x` - Add an x of crossing paths through center.
- `-s` Solver flag. Set maze solving algorithm.
	- `dfs-hunt` - Depth First Search
	- `dfs-gather` - Depth First Search
	- `dfs-corners` - Depth First Search
	- `floodfs-hunt` - Depth First Search
	- `floodfs-gather` - Depth First Search
	- `floodfs-corners` - Depth First Search
	- `rdfs-hunt` - Randomized Depth First Search
	- `rdfs-gather` - Randomized Depth First Search
	- `rdfs-corners` - Randomized Depth First Search
	- `bfs-hunt` - Breadth First Search
	- `bfs-gather` - Breadth First Search
	- `bfs-corners` - Breadth First Search
- `-d` Draw flag. Set the line style for the maze.
	- `sharp` - The default straight lines.
	- `round` - Rounded corners.
	- `doubles` - Sharp double lines.
	- `bold` - Thicker straight lines.
	- `contrast` - Full block width and height walls.
	- `spikes` - Connected lines with spikes.
- `-sa` Solver Animation flag. Watch the maze solution.
	- Any number 1-7. Speed increases with number.
- `-ba` Builder Animation flag. Watch the maze build.
	- Any number 1-7. Speed increases with number.
- `-h` Help flag. Make this prompt appear.

If any flags are omitted, defaults are used.

Examples:

```zsh
cargo run --bin run_maze
cargo run --bin run_maze -- -r 51 -c 111 -b rdfs -s bfs-hunt
cargo run --bin run_maze -- -c 111 -s bfs-gather
cargo run --bin run_maze -- -s bfs-corners -d round -b fractal
cargo run --bin run_maze -- -s dfs-hunt -ba 4 -sa 5 -b wilson-walls -m x
cargo run --bin run_maze -- -h
```

## Settings Detailed

### Row and Column

![dimension-showcase](/images/dimension-showcase.png)

The `-r` and `-c` flags let you set the dimensions of the maze. Note that my programs enforce that rows and columns must be odd, and it will enforce this by incrementing an even value, but this does not affect your usage of the program. As the image above demonstrates, zooming out with `<CTRL-(-)>`, or `CTRL-(+)` allows you to test huge mazes. Note that performance will decrease with size and WSL2 on Windows seems to struggle the most, while MacOS runs quite smoothly regardless of size. Try the `-d contrast` option as pictured if performance is an issue. This options seems to provide smooth performance on all tested platforms.

### Builder Flag

The `-b` flag allows you to specify the algorithm that builds the maze. Maze generation is a deep and fascinating topic because it touches on so many interesting ideas, data structures, and implementation details. I will try to add a writeup for each algorithm in this repository in further detail below.

### Modification Flag

![modification-demo](/images/modification-demo.png)

The `-m` flag places user designated paths in a maze. Most algorithms in the maze generator produce *perfect* mazes. This means that there is a unique path between any two points in the maze, there are no loops, and all locations in the maze are reachable. We can completely ruin this concept by cutting a path through the maze, destroying all walls that lie in the path of our modification. This can create chaotic paths and overlaps between threads.

### Solver Flag

![rdfs-solver-demo](/images/rdfs-solver-demo.png)

The `-s` flag allows you to select the maze solver algorithm. The purpose of this repository is to explore how multithreading can apply to maze algorithms. So far, I have only implemented maze solvers that are multithreading, but I am looking forward to multithreading the maze generation algorithms that would support it. The options are simple for now with breadth and depth first search. However, randomized depth first search can provide interesting results on some maps, like the arena pictured above. As a bonus, breadth first search provides the shortest path for the winning thread, as highlighted in the title image in this repository, when threads are searching for one finish.

An important detail for the solvers is that you can trace the exact path of every thread due to my use of colors. Each thread has a unique color. When a thread walks along a maze path it will leave its color mark behind. If another thread crosses the same path, it will leave its color as well. This creates mixed colors that help you identify exactly where threads have gone in the maze. For depth first searches, I only have the threads paint the path they are currently on, not every square they have visited. This makes it easier to distinguish this algorithm from a breadth first search that paints every seen maze square. If you are looking at static images, not the live animations, the solution you are seeing is a freeze frame of all the threads at the time the game is over: depth first search shows the current position of each thread and the path it took from the start to get there, and breadth first search shows every square visited by all threads at the time a game finishes. Finally, there is `floodfs` solver that is the exact same as a normal depth first search. However, I leave all squares visited by each depth first search colored. This creates a very colorful depth first flooding of the map as threads explore in their respective biased directions. These solvers and their colors create interesting results for the games they play.

![games-showcase](/images/games-showcase.png)

The `hunt` game randomly places a start and a finish then sets the threads loose to see who finds it first.

![hunt-demo](/images/hunt-demo.gif)

The `gather` game forces every thread to find a finish square and will not stop until each has found their own. This is a colorful game to watch with a breadth first search solving algorithm.

![gather-demo](/images/gather-demo.gif)

The `corners` game places each thread in a corner of the maze and they all race to the center to get the finish square. This is a good test to make sure the mazes that the builder algorithms produce are perfect, especially when run with a breadth first search.

![corners-demo](/images/corners-demo.gif)

### Draw Flag

The `-d` flag determines the lines used to draw the maze. The walls are an interesting problem in this project and the way I chose to address walls has allowed me to easily implement both wall adder and path carver algorithms, which I am happy with. Unfortunately, Windows Terminal running WSL2 cannot perfectly connect the horizontal Unicode wall lines, but the result still looks good. MacOS and Linux distributions like PopOS draw everything perfectly and smoothly. You can try all the wall styles out to see which you like the most.

### Animation Flags

The `-ba` flag indicates the speed of the builder animation on a scale from 1-7. The `-sa` flag does the same for the solver animation. This allows you to decide how fast the build or solve process should run. Faster speeds are needed if you zoom out to draw very large mazes.

## Maze Generation Algorithms

When I started this project I was most interested in multithreading the maze solver algorithms. However, as I needed to come up with mazes for the threads to solve I found that the maze generation algorithms are far more interesting. There are even some algorithms in the collection that I think would be well suited for multithreading and I will definitely extend these when I get the chance. For the design of this project I gave myself some constraints and goals. They are as follows.

- No recursion. Many of the maze generation algorithms are recursive. However, I want to be able to produce arbitrarily large mazes as time and memory allows. So, any recursive algorithm must be re-implemented iteratively and produce the same traversals and space complexities as the recursive version. For example, the traditional depth first search generation or solver algorithm must be iterative, produce the same traversal order as a recursive depth first search, and have the same O(D) space complexity in its stack, where D is the current depth/path of the search. This means that the commonly taught depth first search using a stack that pushes all valid neighboring cells onto a stack before proceeding to the next level does not satisfy these constraints.
- Unique maze generation animations. When these algorithms are visualized and animated they should create visuals that are distinct from other generation algorithms. This means that the selection of generators should be broad and have good variety in their implementation details.
- Waste less space. I try my best to use the least amount of space possible, relying on what the maze already provides. Threads already are expensive in terms of space and any data structures they maintain. This means that bit manipulations and encodings are essential to this implementation. Whenever possible we should use the maze itself to store the information we need. I know there is some room for improvement here in many of my generators such as Kruskal's and Prim's. However, I think I have sufficiently met this goal in the recursive depth first search generator and both variations on Wilson's algorithm. Both of these only require O(1) auxiliary space to run.

### Randomized Recursive Depth First Search

![rdfs-loop](/images/rdfs-loop.gif)

This is the classic maze building algorithm. It works by randomly carving out valid walls into passages until it can no longer progress down its current branch. Then it takes one step back and repeats the process. More formally here is the pseudocode.

```txt
mark a random starting point as the origin of all paths

while there are valid maze paths to carve

	mark the current square as visited

	for every neighboring square divided by a wall in random order

		if the neighbor is valid and not seen

			break the wall between current and neighbor

			mark neighbor with the direction you came from

			current becomes next

			continue the outer while loop

	if we are not at the origin

		backtrack to the previous square
```

This is an algorithm that has many optimization possibilities. The most notable actually makes an appearance in the gif that you see above. I encode the directions for backtracking into the bits of a square. In fact, we only need three bits to know how to backtrack and we use them to index into a table with the row and column directions we need to step to. Fortunately for the animation we can also encode those backtracking directions as color coded arrows with those same three bits so you can actually see the markers that we are leaving on the squares. Overall, this is a great algorithm to watch and solve. It is especially fun to watch zoomed out on massive mazes.

### Kruskal's Algorithm

![kruskal-loop](/images/kruskal-loop.gif)

I lump Kruskal, Prim, and Eller together as quite similar but their visual animations are different. Kruskal reasons about the cells and walls in a maze in terms of Disjoint Sets. In fact, I was able to learn about this data structure through this algorithm. I will not go into full detail on what it is, only explain how it is used in this algorithm. The algorithm goes something like this.

```txt
load all square into a disjoint set as single unique sets

shuffle all the walls in the maze randomly

for every wall in the maze

	if the current wall seperates a square above and below

		if a disjoint set union find by rank merges these squares

			break the wall between these squares and join them

	else if the current wall seperates a square left and right

		if a disjoint set union find by rank merges these squares

			break the wall between these squares and join them

```

I am not doing a good job of respecting my space efficiency restriction with this algorithm. I will have to learn more about different approaches because the Disjoint set, walls, and lookup table for squares and their Disjoint set ids takes much space. This is a fun algorithm to watch, however, because of the popping in of maze paths all over the grid.

### Prim's Algorithm

![prim-loop](/images/prim-loop.gif)

There are many versions of Prim's algorithm: simplified, true, and truest are three that I am aware of. I went with true Prim's algorithm and here is how it works.

```txt
load all path cell into a lookup table and give each a random cost

choose a random starting cell

enqueue this cell into a min priority queue by cost

while the min priority queue is not empty

	mark the current cell as visited

	set cell MIN with cost = INFINITY

	for each valid neighbor

		if this neighbor has a lower cost than MIN

			MIN = neighbor

	if MIN is not equal to INFINITY

		break the wall between current and MIN joining squares

		push MIN into the min priority queue

	else

		pop from the min priority

```

This algorithm spreads out nicely as it builds like clusters all over the grid. It is also space inefficient at this time. I hear there are optimizations and will try to learn more.

### Eller's Algorithm

![eller-loop](/images/eller-loop.gif)

People interested in mazes love Eller's algorithm. This is because it can be implemented many ways, some of which allow for arbitrarily large generation of mazes with a memory requirement only equivalent to the width of the maze. This algorithm can generate row by row which makes it quite fast and efficient. For all of its benefits it is challenging to find good information on the implementation. So, I went with a somewhat original approach to the problem for now. I was able to uphold the main benefit of Eller, that being I only require a memory constant tied to the width of the maze. I think there are smarter ways to implement my approach and when I get a chance, I think I can cut down on the number of passes over a row that I require.

```txt
prepare a sliding window of the current and next row

give every cell in the first row of the window a unique set id

for every row in the maze except the last

    give every cell in the next sliding window row a unique set id

    for every column in the current row

        if a square is not part of its right neigbhors set and
        a random choice allows them to be merged

            merge them into the same set, joining squares

    for every set merged with another or left isolated

        choose a random number of elements >= 1 in that set to drop to row below

            join that element with the set below, joining squares.

    adjust the sliding window, set current = next.

for every column in the final row

    if a square is not part of its neighbors set

        merge them into the same set, joining squares.
```

The final row is definitely the trickiest part of this algorithm. However, working it out helps reveal how exact the set tracking must be throughout this algorithm. There are a few key details to consider to notify all cells within a set, and within a row, of a merge.

### Wilson's Path Carver

![wilson-loop](/images/wilson-loop.gif)

The wilson algorithms are my favorite. They are a much more visually interesting and fun to implement approach at producing perfect mazes. This algorithm, as I have implemented it goes as follows.

```txt
pick a random square and make it a path that is part of a maze

pick a random WALK point for a random walk

creat a cell called PREVIOUS that starts as nil.

while we have selected a starting square for a random walk

	for each neighbor NEXT in random order

		if the NEXT != PREVIOUS and is in bounds

			select NEXT for consideration

			if NEXT is part of our own walk

				erase the loop we have formed using backtracking

			else if NEXT is part of the maze

				join our walk to the maze using backtracking to carve a path.

			else

				mark NEXT with the direction it needs for backtracking

				PREVIOUS = WALK

				WALK = NEXT

		continue outer while loop
```

Watching this is one live is very fun and frustrating. Sometimes, you wish the flailing walk path would just find the square sitting out there in the grid but it won't. The starting maze point is a needle in a haystack and we eventually find it but it can just take some time. These are very well balanced mazes with interesting twists and turns and unexpected long paths that can sometimes weave through the maze.

### Wilson's Wall Adder

![wilson-walls-loop](/images/wilson-walls-loop.gif)

While the previous version of Wilson's algorithm is like trying to find a needle in a haystack, we can flip this concept and be the needle surrounded by a haystack. Instead of starting the random walk by trying to find one path point in the maze and then carve a path out when we find it, we can become the walls of the maze. We then surround ourselves with the perimeter walls of the maze and it becomes trivial to find a maze wall. The algorithm is identical. While it is possible it could take a while for a random walk to find a wall at first, in practice this algorithm is extremely fast. I have not done time tests yet, but I am sure that it is much faster than the other version of Wilson's algorithm and can compete with any algorithm discussed so far. Wilson's algorithm is also one that I want to attempt to make multithreaded.

### Randomized Recursive Subdivision

![fractal-loop](/images/fractal-loop.gif)

This algorithm technically produces fractals due to the recursive nature of the technique. It is a recursive algorithm, but I will try to describe the iterative approach. Note that I describe pushing chambers or mazes onto a stack. However, in reality the implementation only needs to push the coordinates of the corner of a chamber, its height, and its width onto a stack, not all the cells. This starts with a maze of all paths and we draw walls.

```txt
push the entire maze onto a stack of chambers

while the stack of chambers is not empty

	if chamber height > chamber width and width meets min requirement

		choose a random height and divide the chamber by that height

		choose a random point in the divide for a path gap

		update the current chamber's height

		push the chamber after the divide onto the stack

	else if chamber width >= chamber height and height meets min requirement

		choose a random width and divide the chamber by that width

		choose a random point in the divide for a path gap

		update the current chamber's width

		push the chamber after the divide onto the stack

	else

		pop chamber from the stack.

```

This is a great algorithm because the mazes it produces are completely different from anything esle that you see. I appreciate the interesting flow patterns that breadth first searches produce.

### Randomized Grid

![grid-loop](/images/grid-loop.gif)

This algorithm is my own addition to the repository because I wanted something chaotic for the threads to race through. This algorithm is a modified recursive depth first search. It works as follows.

```txt
mark a random starting point and push it onto a stack

set LIMIT to be the maximum length to travel in one direction

while the stack is not empty

	mark the CURRENT square as visited

	for every neighboring square divided by a wall in random order

		if the neighbor NEXT is valid and not seen

			while run is less than LIMIT and CURRENT is valid

				break wall between CURRENT and NEXT

				mark NEXT as visited

				push next onto the stack

				CURRENT becomes NEXT

			continue outer loop

	if no neighbor was found

		pop from the stack
```

### Arena

![arena-loop](/images/arena-loop.gif)

This is just a simple arena of paths. Try different solver algorithms to see some pretty colors.

## References

- *Mazes for Programmers* by Jamis Buck was a great starting point for many of the core ideas behind the algorithms that build these mazes. However, writing in Rust required me to often take a different approaches than those used by Buck.
- The [Maze Generation Algorithm](https://en.wikipedia.org/wiki/Maze_generation_algorithm) Wikipedia page is very helpful in outlining some pseudocode for most of these algorithms.
