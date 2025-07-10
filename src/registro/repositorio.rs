pub trait RegistroRepo {
  fn agregar(&self, reg: &Registro);
}

impl RegistrRepo for RegistroMySQL {
  fn agregar(&self, reg: &Registro) {
    // Aquí iría la lógica para agregar el registro a la base de datos MySQL
    println!("Registro agregado: {:?}", reg);
  }
}
