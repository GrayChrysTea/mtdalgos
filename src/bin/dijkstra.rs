use std::{
    io::Error,
    process::exit,
    //thread::sleep,
    //time::Duration,
};

use mtdalgos::dijkstra::simple::{AdjacencyMatrix, NodeWithCost, MtdDijkstra};

macro_rules! pushtomatrix {
    ($matrix: expr, $from: expr, $to: expr, $cost: expr) => {
        $matrix.push($from, NodeWithCost::new($to, $cost))?;
    };
}

fn run() -> Result<(), Error> {
    //let delay = 500;
    //println!("Running dijkstra with delay of {}ms.", delay);
    let mut matrix = AdjacencyMatrix::new(5);
    pushtomatrix!(matrix, 0, 1, 1);
    pushtomatrix!(matrix, 1, 2, 1);
    pushtomatrix!(matrix, 2, 3, 2);
    pushtomatrix!(matrix, 3, 4, 1);
    pushtomatrix!(matrix, 2, 4, 6);
    let mut processor = MtdDijkstra::new(3, 5, matrix)?;
    processor.calculate()?;
    //sleep(Duration::from_millis(delay));
    for node in 0..5 {
        println!("{}: {:?}", node, processor.get(node));
    }
    return Ok(());
}

fn main() {
    match run() {
        Ok(_) => exit(0),
        Err(error) => {
            println!("{:?}", error);
            exit(1)
        },
    }
}
