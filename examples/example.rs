extern crate time;
extern crate core;
extern crate simple_serialize;

use std::default::Default;

use simple_serialize::{SimpleSerialize, SimpleSerializeVec};

fn main()
{
    println!("Encoding/decoding throughput for {{ uint, (uint, (uint, uint)), Vec<Vec<uint>, and Option<uint> }}.");
    println!("Caveat: compiler optimizations from whole program analysis; actual performance should be worse.");

    test_simpleser(1024, |i| i);
    test_simpleser(1024, |i| (i, (i+1, i-1)));
    test_simpleser(128, |_| vec![vec![0u, 1u], vec![1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2]]);
    test_simpleser(1024, |i| if i % 2 == 0 { Some(i) } else { None });
}

// bounces some elements back and forth between columnar stacks, encoding/decoding ...
fn test_simpleser<T: SimpleSerialize<R>, R: SimpleSerializeVec<T>>(number: uint, element: |uint|:'static -> T)
{
    let start = time::precise_time_ns();

    let mut stack1: R = Default::default();
    let mut stack2: R = Default::default();

    let mut buffers = Vec::new();

    for index in range(0, number) { stack1.push(element(index)); }
    stack1.encode(&mut buffers);


    let mut bytes = 0u;     // number of bytes per iteration
    let mut total = 0u;     // total bytes processed

    for buffer in buffers.iter() { bytes += buffer.len(); }

    while time::precise_time_ns() - start < 1000000000
    {
        // decode, move, encode
        stack1.decode(&mut buffers);
        while let Some(record) = stack1.pop() { stack2.push(record); }
        stack2.encode(&mut buffers);

        // decode, move, encode
        stack2.decode(&mut buffers);
        while let Some(record) = stack2.pop() { stack1.push(record); }
        stack1.encode(&mut buffers);

        total += 2 * bytes;
    }

    println!("Encoding/decoding at {}GB/s", total as f64 / (time::precise_time_ns() - start) as f64)
}
