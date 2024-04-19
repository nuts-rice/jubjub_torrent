pub fn chi_squared_test(observed: &Vec<f64>, expected: &Vec<f64>) -> f64 {
    let mut chi_squared = 0.0;
    for i in 0..observed.len() {
        chi_squared += (observed[i] - expected[i]).powi(2) / expected[i];
    }
    chi_squared
}
