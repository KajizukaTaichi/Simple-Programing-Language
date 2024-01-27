use crate::executor::*;

impl<'a> Executor<'a> {
    pub fn declaration_function(&mut self, function: Function) {
        let name = function.name.clone();
        let index = self.name_space.len();
        self.name_space.push(function);
        self.log_print(format!("メモリに関数を保存しました"));
        let value = Type::Function(index);
        self.set_variable(name, value);
    }

    pub fn function_propaty(&self, pointer: usize) -> &Function {
        &self.name_space[pointer]
    }

    pub fn execute_function(&mut self, pointer: usize, args: Vec<Type>) -> ReturnValue {
        let func = self.function_propaty(pointer);
        let code = func.code.clone();
        let arg_names = func.args.clone();

        dbg!(args.clone());

        let mut pre = Vec::new();

        for (i, j) in arg_names.iter().zip(args) {
            match j {
                Type::String(s) => {
                    pre.push(format!("var {i} = '{s}'")); // 引数は変数として扱われる
                }
                Type::Number(f) => pre.push(format!("var {i} = {f}")),
                Type::List(l) => pre.push(format!(
                    "var {i} = list({})",
                    l.iter()
                        .map(|x| match x {
                            Type::Number(i) => i.to_string(),
                            Type::String(s) => format!("'{s}'"),
                            Type::List(_) => "".to_string(),
                            Type::Bool(b) => {
                                b.to_string()
                            }
                            Type::Function(i) => self.function_propaty(*i).name.clone(),
                        })
                        .collect::<Vec<String>>()
                        .join(", ")
                )),
                Type::Bool(b) => {
                    pre.push(format!("var {i} = {}", &b.to_string()));
                }
                Type::Function(i) => {
                    pre.push(format!("var {i} = {}", self.function_propaty(i).name))
                }
            }
        }

        let mut instance = Executor::new(
            &mut self.memory,
            &mut self.name_space,
            ExecutionMode::Script,
        );

        instance.execute_block(pre);
        instance.execution_mode = self.execution_mode.clone();

        if let ExecutionMode::Interactive = instance.execution_mode {
            instance.execution_mode = ExecutionMode::Debug
        }
        return instance.execute_block(code);
    }
}
