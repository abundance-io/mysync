/*
implementation of myers approach to diffing between files with dijkstra's algorithm for graph traversal
taking the individual bytes of the file as units
*/

use rayon::{prelude::*, slice::ChunksMut};

fn con_range(range: usize, val: usize) -> Option<usize> {
    //replace all values not in range with range+1
    if val > range - 1 {
        None
    } else {
        Some(val)
    }
}

#[derive(Clone)]
struct Node {
    //an index to bytes in the source and destination
    //with source as x axis and destination as y (myers)  ---> (source_index,destination_index)
    index: usize,
    //using vector indices to identify connected nodes
    connected: Vec<usize>,
}

struct MyersGraph<T: Eq + Sync >{

    source: Vec<T>,
    destination: Vec<T>,
    nodes: Vec<Node>,
}

#[derive(Clone, PartialEq)]
struct DijkstraWrapper {
    node: usize,
    distance: usize,
    finished: bool,
    from: Option<usize>,
}

impl<T:Eq + Sync >MyersGraph<T>{
    fn new( source: Vec<T>, destination: Vec<T>) -> Self {
        
        //first pass create all the nodes without links
        let mut nodes = vec![];
        let graph_size = (source.len() + 1) * (destination.len() + 1);

        for count in 0..graph_size {
            let mut connected = vec![
                count - 1,            //prev node
                count + 1,            //next node
                count + source.len(), //node_below
                count - source.len(), //node above
            ];

            connected = connected
                .iter()
                .filter_map(|item| con_range(source.len(), item.clone()))
                .collect();

            nodes.push(Node {
                index: count,
                connected: connected,
            });
        }

        //add skip connections for all similar bytes (with parallel iteration )
        nodes.par_chunks_mut(1).for_each(|chunk| {
            let (x, y) = (chunk[0].index / source.len(), chunk[0].index % source.len());
            if x < source.len() {
                if source[x] == destination[y] {
                    //add skip connection from current source node to next destination node
                    chunk[0].connected.push(chunk[0].index + source.len() + 1)
                }
            }
        });

        //to complete graph iterate through all the nodes and fully link the connected nodes
        for node in nodes.clone() {
            for connected_node in node.connected {
                if !nodes[connected_node].connected.contains(&node.index) {
                    nodes[connected_node].connected.push(node.index)
                }
            }
        }

        Self {
            source: source,
            destination: destination,
            nodes: nodes,
        }
    }

    ///Find the shortest path between two nodes on the myers graph using Dijkstra's algorithm
    fn shortest(self, source_node: usize, destination_node: usize) -> Vec<usize> {
        let mut visited = vec![];
        visited.push(DijkstraWrapper {
            node: source_node,
            finished: false,
            distance: 0,
            from: None,
        });

        while visited.len() < self.source.len() {
            //visit all the nodes
            let mut unfinished_nodes: Vec<(usize, DijkstraWrapper)> = visited
                .iter()
                .cloned()
                .enumerate()
                .filter(|x| !x.1.finished)
                .collect();

            unfinished_nodes.sort_unstable_by_key(|x| x.1.distance);

            let (index, curr_node) = &unfinished_nodes[0];

            for node in &self.nodes[curr_node.node].connected {
                //check if node has already been visited
                if (!visited.iter().any(|x| x.node == *node)) {
                    visited.push(
                        //nodes all weigh the same unit
                        //todo -- add biases towards insertion or deletion
                        DijkstraWrapper {
                            node: node.clone(),
                            distance: curr_node.distance + 1,
                            finished: false,
                            from: Some(curr_node.node),
                        },
                    )
                }
            }

            visited[index.clone()].finished = true;
        }

        let dest_point = visited
            .iter()
            .cloned()
            .find(|x| x.node.clone() == destination_node)
            .unwrap();
        let mut shortest_vec: Vec<usize> = vec![dest_point.node.clone()];
        let mut next_point = dest_point;
        while next_point.node != source_node {
            let next_node = &self.nodes[next_point.from.unwrap()].clone();
            shortest_vec.push(next_node.index.clone());

            next_point = visited
                .iter()
                .cloned()
                .find(|x| x.node == next_point.from.unwrap())
                .unwrap();
        
            }
        shortest_vec.push(source_node);
        return shortest_vec.into_iter().rev().collect();
    }

    fn construct_changes(self,diff_path:Vec<usize>) -> Vec<String>{
   

       todo!() 
    }
}
#[cfg(test)]
mod tests {
    use super::MyersGraph;


    #[test]
    fn test_shortest(){
        let source = Vec::from_iter(String::from("abundance").chars());
        let destination = Vec::from_iter(String::from("abundy").chars());
        let mut edit_graph = MyersGraph::new(source,destination);
        let node_length = &edit_graph.nodes.len();
        let edit_path = &edit_graph.shortest(0,node_length.clone());    
        println!("{:?}",*edit_path);

        let a:usize = 3; 
    }
    fn testread() {}
}
