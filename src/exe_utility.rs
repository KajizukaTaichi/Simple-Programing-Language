use crate::executor::{Executor, Type};

impl<'a> Executor<'a> {
    /// メモリを表示
    pub fn show_memory(&mut self) {
        let mut name_max_len = 0;
        for i in self.memory.iter() {
            if name_max_len < i.name.len() {
                name_max_len = i.name.len()
            }
        }

        let mut value_max_len = 0;
        for item in self.memory.iter() {
            if let Type::Number(i) = item.value {
                if value_max_len < i.to_string().len() {
                    value_max_len = i.to_string().len()
                }
            }
        }

        if !self.memory.is_empty() {
            self.log_print(format!("+-- メモリ内の変数"));
            let len = self.memory.len();
            for index in 0..len {
                let vars = self.memory[index].clone();
                let value = self.type_string(vars.value.clone());
                println!("| [{:>3}] {:<name_max_len$} :{}", index, vars.name, value);
            }
        } else {
            self.log_print(format!("変数がありません"));
        }
    }
}
