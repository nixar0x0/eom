
extern crate ndarray;
extern crate ndarray_odeint;
extern crate ndarray_linalg;
extern crate itertools;
extern crate num_traits;

use std::fs::*;
use std::io::Write;
use ndarray::*;
use ndarray_linalg::prelude::*;
use ndarray_odeint::prelude::*;
use itertools::iterate;
use num_traits::int::PrimInt;

macro_rules! impl_accuracy {
    ($name:ident, $method:path, $filename:expr) => {
fn $name() {
    let data: Vec<_> = (0..12)
        .map(|n| {
            let dt = 0.1 / 2.pow(n) as f64;
            let eom = Lorenz63::default();
            let teo = $method(eom, dt);
            let t = 100 * 2.pow(n);
            let ts = iterate(rcarr1(&[1.0, 0.0, 0.0]), |y| teo.iterate(y.clone()));
            (dt, ts.take(t+1).last().unwrap())
        })
        .collect();

    let mut f = File::create($filename).unwrap();
    write!(&mut f, "dt,dev\n").unwrap();
    for i in 0..(data.len() - 1) {
        let dt = data[i].0;
        let dev = (&data[i + 1].1 - &data[i].1).norm();
        write!(&mut f, "{},{}\n", dt, dev).unwrap();
    }
}
}} // impl_accuracy_test

impl_accuracy!(euler, explicit::euler, "euler.csv");
impl_accuracy!(heun, explicit::heun, "heun.csv");
impl_accuracy!(rk4, explicit::rk4, "rk4.csv");

fn main() {
    euler();
    heun();
    rk4();
}
