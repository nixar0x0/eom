//! Calculate Covariant Lyapunov Vector
//!
//! This function consumes much memory since this saves matrices duraing the time evolution.

extern crate ndarray;
extern crate ndarray_odeint;
extern crate ndarray_linalg;

use std::io::Write;
use ndarray::*;
use ndarray_linalg::*;
use ndarray_odeint::*;
use std::mem::replace;

fn clv_backward(c: &Array2<f64>, r: &Array2<f64>) -> (Array2<f64>, Array1<f64>) {
    let cd = r.solve_triangular(UPLO::Upper, ::ndarray_linalg::Diag::NonUnit, c)
        .expect("Failed to solve R");
    let (c, d) = normalize(cd, NormalizeAxis::Column);
    let f = Array::from_vec(d).mapv_into(|x| 1.0 / x);
    (c, f)
}

pub fn clv<TEO>(teo: &TEO,
                x0: Array1<f64>,
                alpha: f64,
                duration: usize)
                -> Vec<(Array1<f64>, Array2<f64>, Array1<f64>)>
    where TEO: TimeStep<f64>,
          for<'a> &'a TEO: TimeEvolution<OwnedRepr<f64>, Ix1>,
          for<'a, 'b> Jacobian<'a, f64, OwnedRepr<f64>, Ix1, TEO>: OperatorMut<ViewRepr<&'b mut f64>, Ix1>
{
    let n = x0.len();
    let ts = time_series(x0, &teo);
    let qr_series = ts.scan(Array::eye(n), |q, x| {
        let (q_next, r) = jacobian(teo, x.clone(), alpha)
            .op_multi_mut(q)
            .qr()
            .unwrap();
        let q = replace(q, q_next);
        Some((x, q, r))
    }).skip(duration / 10)
        .take(duration + duration / 10)
        .collect::<Vec<_>>();
    let clv_rev = qr_series
        .into_iter()
        .rev()
        .scan(Array::eye(n), |c, (x, q, r)| {
            let (c_now, f) = clv_backward(c, &r);
            let v = q.dot(&c_now);
            *c = c_now;
            Some((x, v, f))
        })
        .collect::<Vec<_>>();
    clv_rev.into_iter().skip(duration / 10).rev().collect()
}

fn main() {
    let dt = 0.01;
    let eom = model::Lorenz63::default();
    let teo = explicit::rk4(eom, dt);
    let duration = 100000;
    let ts = clv(&teo, arr1(&[1.0, 0.0, 0.0]), 1e-7, duration);
    let mut l = Array::zeros(3);
    println!("v0v1, v0v2, v1v2");
    for (_, v, f) in ts.into_iter().rev() {
        let v0 = v.axis_iter(Axis(1)).nth(0).unwrap();
        let v1 = v.axis_iter(Axis(1)).nth(1).unwrap();
        let v2 = v.axis_iter(Axis(1)).nth(2).unwrap();
        println!("{}, {}, {}", v0.dot(&v1), v0.dot(&v2), v1.dot(&v2));
        l += &f.map(|x| x.abs().ln());
    }
    write!(&mut std::io::stderr(),
           "exponents = {:?}\n",
           l / (dt * duration as f64))
        .unwrap();
}
