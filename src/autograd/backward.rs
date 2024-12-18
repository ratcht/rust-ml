use super::{Graph, Value};  // Import Graph from parent module
// scalar.rs
use std::cell::{Cell, RefCell};
use std::collections::HashSet;

pub trait Backward {
  fn add_backward(&self, lhs: usize, rhs: usize, grad: f32);
  fn mul_backward(&self, lhs: usize, rhs: usize, grad: f32);
  fn div_backward(&self, lhs: usize, rhs: usize, grad: f32);
  fn exp_backward(&self, lhs: usize, grad: f32);
  fn backward(&self, id: usize);
  fn backward_from(&self, idx: usize);


}

impl Backward for Graph {
  // d(a + b)/da = 1, d(a + b)/db = 1
  fn add_backward(&self, lhs: usize, rhs: usize, grad: f32) {
    let values = self.values.borrow();
    let lhs_value = &values[lhs];
    let rhs_value = &values[rhs];
    lhs_value.grad.set(lhs_value.grad.get() + grad);
    rhs_value.grad.set(rhs_value.grad.get() + grad);
  }

  // d(a * b)/da = b, d(a * b)/db = a
  fn mul_backward(&self, lhs: usize, rhs: usize, grad: f32) {
    let values = self.values.borrow();
    let lhs_value = &values[lhs];
    let rhs_value = &values[rhs];
    lhs_value.grad.set(lhs_value.grad.get() + rhs_value.data * grad);
    rhs_value.grad.set(rhs_value.grad.get() + lhs_value.data * grad);
  }

  // d(a/b)/da = 1/b, d(a/b)/db = (-a/b**2)
  fn div_backward(&self, lhs: usize, rhs: usize, grad: f32) {
    let values = self.values.borrow();
    let lhs_value = &values[lhs];
    let rhs_value = &values[rhs];
    lhs_value.grad.set(lhs_value.grad.get() + (1./rhs_value.data) * grad);
    rhs_value.grad.set(rhs_value.grad.get() + (-lhs_value.data/(rhs_value.data*rhs_value.data)) * grad);
  }


  // d(e^x)/dx = e^x
  fn exp_backward(&self, lhs: usize, grad: f32) {
    let values = self.values.borrow();
    let lhs_value = &values[lhs];
    let exp_x = f32::exp(lhs_value.data);
    lhs_value.grad.set(lhs_value.grad.get() + grad * exp_x);
  }


  fn backward(&self, id: usize) {
    self.values.borrow()[id].grad.set(1.0);

    // Topo sort
    let mut topo = vec![];
    let mut visited = HashSet::new();

    fn build_topo(v: usize, values: &RefCell<Vec<Value>>, visited: &mut HashSet<usize>, topo: &mut Vec<usize>) {
      if !visited.contains(&v) {
        visited.insert(v);

        let children = {
          let values = values.borrow();
          values[v].prev.clone()
        };

        for &child in &children {
          build_topo(child, values, visited, topo);
        }

        topo.push(v);
      }
    }

    build_topo(id, &self.values, &mut visited, &mut topo);
    
    for &idx in topo.iter().rev(){
      self.backward_from(idx);
    }

   // 
  }

  fn backward_from(&self, idx: usize) {
    let values = self.values.borrow();
    let value = &values[idx];
    let grad = value.grad.get();

    match value.op.as_str() {
      "+" => {
        let lhs = value.prev[0];
        let rhs = value.prev[1];
        self.add_backward(lhs, rhs, grad);
      },
      "*" => {
        let lhs = value.prev[0];
        let rhs = value.prev[1];
        self.mul_backward(lhs, rhs, grad);
      },
      "/" => {
        let lhs = value.prev[0];
        let rhs = value.prev[1];
        self.div_backward(lhs, rhs, grad);
      },
      "exp" => {
        let lhs = value.prev[0];
        self.exp_backward(lhs, grad);
      },
      
      _ => {}
    }
  }

}