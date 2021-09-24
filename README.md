# Matrix Multiplication Biclique Compression


Input is an [ASCIIGraph](https://webgraph.di.unimi.it/docs/it/unimi/dsi/webgraph/ASCIIGraph.html) of the WebGraph framework.
The first step is to construct a `CeciliaGraph`, which is a binary format of the input graph.
It consists of:

 - number of total nodes : 32-bit
 - number of total edges : 32-bit
 - for each node `v` in the adjacency lists:
   - node id `v` : 32-bit, negated
   - all nodes of the adjacency list of `v`, each 32-bit

`cargo run --release --bin ceciliatxt2ceciliabin -- -i ASCIIGraphFileName -o CeciliaGraphFileName`

Next, we run [Cecilia's `vnmextract` program](https://github.com/koeppl/biclique_extraction) that creates files for the extracted bicliques and files for the remaining nodes:

`./vnmextract $i 1 1 100 500,100 ${basename} 4`

where `${basename}` is the filename used as a prefix for the generated files.
The most recent file containing all remaining nodes is 

`ls --sort time biclique/${basename}-it-* | head -n1 > RemainingGraph`

The bicliques can be concatenated into a single file 

`cat "$kBicliqueDir/${basename}-biclique"*.txt > BiCliqueFile`
.

We can run

`cargo run --release --bin ceciliatxt2ceciliabin -- -i RemainingGraph -o RemainingCeciliaGraph`

to generate a CeciliaGraph from the remaining nodes.

Finally, we can 

- run a PageRank benchmark on the original graph

`cargo run --release --bin ceciliabin2asciigraph -- -i CeciliaGraphFileName`

- run a PageRank benchmark on the remaining graph with the bicliques
 `cargo run --release --bin ceciliabin2asciigraph -- -c BiCliqueFile -i RemainingCeciliaGraph`
- run a PageRank benchmark on the remaining graph with the bicliques and use the original graph to correct missing self-loops or self-loops that have been introduced
 `cargo run --release --bin ceciliabin2asciigraph -- -c BiCliqueFile -i RemainingCeciliaGraph -g CeciliaGraphFileName`
- restore the original ASCIIGraph (modulo self loops) via
 `cargo run --release --bin ceciliabin2asciigraph -- -c BiCliqueFile -i RemainingCeciliaGraph -o newASCIIGraphFileName`


Caveats:

- The biclique extraction tool introduces self-loops, which we remove in the remaining nodes.
Hence, we may remove self-loops present in the original graph, or might introduce self-loops that are indirectly given by a biclique.
Therefore, we provide the program argument `-g` to remove them by an additional pass on the original graph.
