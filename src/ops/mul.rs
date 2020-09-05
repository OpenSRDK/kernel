use crate::Kernel;
use rayon::prelude::*;
use std::ops::Mul;

impl<T> Mul<Kernel<T>> for Kernel<T>
where
    T: 'static + ?Sized,
{
    type Output = Self;

    fn mul(self, rhs: Kernel<T>) -> Self::Output {
        let self_params_len = self.params.len();

        let params = [&self.params as &[f64], &rhs.params as &[f64]].concat();

        let self_func = self.func;
        let rhs_func = rhs.func;

        Self {
            params,
            func: Box::new(move |x: &T, x_prime: &T, with_grad: bool, params: &[f64]| {
                let (fx, dfx) = self_func(x, x_prime, with_grad, &params[..self_params_len])?;
                let (gx, dgx) = rhs_func(x, x_prime, with_grad, &params[self_params_len..])?;

                let func = fx * gx;

                let grad = if !with_grad {
                    None
                } else {
                    let grad = dfx
                        .unwrap()
                        .par_iter()
                        .map(|dfxi| dfxi * gx)
                        .chain(dgx.unwrap().par_iter().map(|dgxi| fx * dgxi))
                        .collect::<Vec<_>>();

                    Some(grad)
                };

                Ok((func, grad))
            }),
        }
    }
}
