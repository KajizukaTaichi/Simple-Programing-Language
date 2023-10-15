use rand::Rng;
use std::io::{self, Write};
use std::process::exit;

fn input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut result = String::new();
    io::stdin().read_line(&mut result).ok();
    return result
        .trim_start()
        .trim_end()
        .parse()
        .ok()
        .unwrap_or("".to_string());
}

/// 変数のデータ
#[derive(Clone)]
pub struct Variable {
    name: String,
    value: f64,
    expr: String,
}

/// 関数のデータ
#[derive(Clone)]
pub struct Func {
    name: String,
    code: String,
}

/// 変数の重複を削除する
fn remove_duplicates_variable(memory: &mut Vec<Variable>) -> &mut Vec<Variable> {
    let mut seen_names = std::collections::HashMap::new();
    let mut to_remove = Vec::new();

    for (index, memory) in memory.iter().enumerate() {
        if let Some(existing_index) = seen_names.get(&memory.name) {
            to_remove.push(if existing_index < &index {
                *existing_index
            } else {
                index
            });
        } else {
            seen_names.insert(&memory.name, index);
        }
    }

    to_remove.sort(); // Sort indices in ascending order

    for (i, index) in to_remove.iter().enumerate() {
        memory.remove(index - i); // Adjust for removed items before
    }
    return memory;
}

/// 関数の重複を削除する
fn remove_duplicates_function(memory: &mut Vec<Func>) -> &mut Vec<Func> {
    let mut seen_names = std::collections::HashMap::new();
    let mut to_remove = Vec::new();

    for (index, memory) in memory.iter().enumerate() {
        if let Some(existing_index) = seen_names.get(&memory.name) {
            to_remove.push(if existing_index < &index {
                *existing_index
            } else {
                index
            });
        } else {
            seen_names.insert(&memory.name, index);
        }
    }

    to_remove.sort(); // Sort indices in ascending order

    for (i, index) in to_remove.iter().enumerate() {
        memory.remove(index - i); // Adjust for removed items before
    }
    memory
}

/// コードを実行を管理
pub struct Executor {
    memory: Vec<Variable>, //メモリの変数
    name_space: Vec<Func>, // 関数
    stmt: String,          // ブロックのステートメント
    else_stmt: String,     // elseステートメント
    count: usize,          // ループカウンタ
    name: String,          // 関数の名前
    expr: String,          // 条件式
    mode: String,          // 制御ブロックの状態
    old_mode: String,      // 元のモード
    nest_if: usize,        // ifネストの階層を表す
    nest_for: usize,       // forネストの階層を表す
    nest_while: usize,     // whileネストの階層を表す
    nest_func: usize,      // funcネストの階層を表す
}

impl Executor {
    pub fn new(memory: &Vec<Variable>, name_space: &Vec<Func>) -> Executor {
        Executor {
            memory: memory.to_owned(),
            name_space: name_space.to_owned(),
            stmt: "".to_string(),
            else_stmt: "".to_string(),
            count: 0,
            name: "".to_string(),
            expr: "".to_string(),
            mode: "normal".to_string(),
            old_mode: "normal".to_string(),
            nest_if: 0,
            nest_for: 0,
            nest_while: 0,
            nest_func: 0,
        }
    }

    pub fn execute(&mut self, code: String) -> Option<f64> {
        let lines = code.trim_start().trim_end();
        if self.mode == "for".to_string() {
            if lines.find("end for").is_some() {
                if self.nest_for > 0 {
                    self.nest_for -= 1;
                    self.stmt += lines;
                    self.stmt += "\n";
                } else {
                    for _ in 0..self.count {
                        let status =  Executor::new(&self.memory, &self.name_space).execute_block(&self.stmt);
                        match status {
                            Some(i) => {
                                if i == f64::MAX {
                                    //状態が1(break)の時はループを抜け出す
                                    break;
                                } else {
                                    return Some(i);
                                }
                            }
                            None => {}
                        }
                    }
                    self.stmt = String::new();
                    self.mode = self.old_mode.clone();
                }
            } else if lines.contains("for") {
                self.nest_for += 1;
                self.stmt += lines;
                self.stmt += "\n";
            } else {
                self.stmt += lines;
                self.stmt += "\n";
            }
        } else if self.mode == "if".to_string() {
            if lines.find("else").is_some() {
                self.old_mode = self.mode.clone();
                self.mode = "else".to_string()
            } else if lines.find("end if").is_some() {
                if self.nest_if > 0 {
                    self.nest_if -= 1;
                    self.stmt += lines;
                    self.stmt += "\n";
                } else {
                    if self.calculation(self.expr.clone()) != 0.0{
                        let status = Executor::new(&self.memory, &self.name_space).execute_block(&self.stmt);
                        match status {
                            Some(i) => {
                                if i == f64::MAX {
                                    return Some(f64::MAX);
                                } else {
                                    return Some(i);
                                }
                            }
                            None => {}
                        }
                        self.stmt = String::new();
                    } else {
                        self.stmt = String::new();
                    }
                    self.mode = self.old_mode.clone();
                }
            } else if lines.find("if").is_some() {
                self.nest_if += 1;
                self.stmt += lines;
                self.stmt += "\n";
            } else {
                self.stmt += lines;
                self.stmt += "\n";
            }
        } else if self.mode == "func".to_string() {
            if lines.contains("end func") {
                if self.nest_func > 0 {
                    self.nest_func -= 1;
                } else {
                    self.name_space.push(Func {
                        name: self.name.clone(),
                        code: self.stmt.clone(),
                    });
                    self.stmt = String::new();
                    self.mode = self.old_mode.clone();
                }
            } else if lines.find("func").is_some() {
                self.nest_func += 1;
            } else {
                self.stmt += lines;
                self.stmt += &String::from("\n");
            }
        } else if self.mode == "else".to_string() {
            if lines.find("end if").is_some() {
                if self.nest_if > 0 {
                    self.nest_if -= 1;
                    self.stmt += lines;
                    self.stmt += "\n";
                } else {
                    if self.calculation(self.expr.clone()) == 0.0 {
                        let status =  Executor::new(&self.memory, &self.name_space).execute_block(&self.else_stmt);
                        match status {
                            Some(i) => {
                                if i == f64::MAX {
                                    return Some(f64::MAX);
                                } else {
                                    return Some(i);
                                }
                            }
                            None => {}
                        }
                        self.else_stmt = String::new();
                        self.stmt = String::new();
                    } else {
                        let status =  Executor::new(&self.memory, &self.name_space).execute_block(&self.stmt);
                        match status {
                            Some(i) => {
                                if i == f64::MAX {
                                    return Some(f64::MAX);
                                } else {
                                    return Some(i);
                                }
                            }
                            None => {}
                        }
                        self.else_stmt = String::new();
                        self.stmt = String::new();
                    }
                    self.mode = self.old_mode.clone();
                }
            } else if lines.find("if").is_some() {
                self.nest_if += 1;
                self.stmt += lines;
                self.stmt += "\n";
                self.mode = "if".to_string();
            } else {
                self.else_stmt += lines;
                self.else_stmt += &String::from("\n");
            }
        } else if self.mode == "while".to_string() {
            if lines.find("end while").is_some() {
                if self.nest_while > 0 {
                    self.nest_while -= 1;
                } else {
                    loop {
                        if self.calculation(self.expr.clone()) == 0.0 {
                            self.stmt = String::new();
                            break;
                        } else {
                            let status =  Executor::new(&self.memory, &self.name_space).execute_block(&self.stmt);
                            match status {
                                Some(i) => {
                                    if i == f64::MAX {
                                        //状態が1(break)の時はループを抜け出す
                                        break;
                                    } else {
                                        return Some(i);
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                    self.mode = self.old_mode.clone();
                }
            } else if lines.find("while").is_some() {
                self.nest_while += 1;
            } else {
                self.stmt += lines;
                self.stmt += "\n";
            }
        } else {
            if lines.find("var").is_some() {
                let new_lines = lines.replacen("var", "", 1);
                let lines = &new_lines;
                let params: Vec<&str> = lines.split("=").collect();
                let value = self.calculation(params[1..].join("=").to_string());
                self.memory.push(Variable {
                    name: params[0].trim().replace(" ", ""),
                    value,
                    expr: params[1..].join("=").to_string(),
                });
            } else if lines.find("calc").is_some() {
                let new_lines = lines.replacen("calc", "", 1);
                let name = &new_lines;
                match self.reference_variable(name.to_string()) {
                    Some(index) => {
                        let value = self.calculation(self.memory[index].to_owned().expr);
                        self.memory[index].value = value;
                    }
                    None => {}
                }
            } else if lines.find("func").is_some() {
                let new_lines = lines.trim().replacen("func", "", 1).replace(" ", "");
                self.name = new_lines.replace(" ", "");
                self.mode = "func".to_string();
            } else if lines.find("call").is_some() {
                let new_lines = lines.replacen("call", "", 1);
                let name = &new_lines.replace(" ", "");
                match self.reference_function(name.clone()) {
                    Some(index) => {
                         Executor::new(&self.memory, &self.name_space).execute_block(&self.name_space[index].code);
                    }
                    None => {}
                }
            } else if lines.find("for").is_some() {
                let new_lines = lines.replacen("for", "", 1);
                self.count = self.calculation(new_lines) as usize;
                self.old_mode = self.mode.clone();
                self.mode = "for".to_string();
            } else if lines.find("if").is_some() {
                let new_lines = lines.replacen("if", "", 1);
                self.expr = new_lines;
                self.old_mode = self.mode.clone();
                self.mode = "if".to_string()
            } else if lines.find("while").is_some() {
                let new_lines = lines.replacen("while", "", 1);
                self.expr = new_lines;
                self.old_mode = self.mode.clone();
                self.mode = "while".to_string();
            } else if lines.find("input").is_some() {
                let new_lines = lines.replacen("input", "", 1);
                let name = &new_lines;
                let inputed = input("> ");
                let value = self.calculation(inputed.clone());
                self.memory.push(Variable {
                    name: name.to_owned(),
                    value: value,
                    expr: inputed,
                });
            } else if lines.find("print").is_some() {
                let new_lines = lines.replacen("print", "", 1);
                let mut text = String::new();
                let params = &new_lines;
                for i in params.split(",").collect::<Vec<&str>>() {
                    if i.find("'").is_some() || i.find("\"").is_some() {
                        //文字列か？
                        text += &i.replace("'", "").replace("\"", "");
                    } else {
                        //文字列以外は式として扱われる
                        text += self.calculation(i.trim().to_string()).to_string().as_str();
                    }
                }
                println!("{text}");
            } else if lines.find("del").is_some() {
                let new_lines = lines.replacen("del", "", 1);
                let name = &new_lines;
                match self.reference_variable(name.clone()) {
                    Some(index) => {
                        self.memory.remove(index);
                    }
                    None => {}
                }
                match self.reference_function(name.to_owned()) {
                    Some(index) => {
                        self.memory.remove(index);
                    }
                    None => {}
                }
            } else if lines.find("#").is_some() {
            } else if lines.find("rand").is_some() {
                let new_lines = lines.replacen("rand", "", 1);
                let params = new_lines.split(",").collect::<Vec<&str>>();
                if params.len() < 3 {
                    let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                    let temp: i64 = rng.gen_range(1, 10);
                    self.memory.push(Variable {
                        name: params[0].trim().replace(" ", ""),
                        value: temp as f64,
                        expr: temp.to_string(),
                    });
                } else {
                    let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                    let temp: i64 = rng.gen_range(
                        self.calculation(String::from(params[1])).round() as i64,
                        self.calculation(String::from(params[2])).round() as i64,
                    );
                    self.memory.push(Variable {
                        name: params[0].trim().replace(" ", ""),
                        value: temp as f64,
                        expr: temp.to_string(),
                    });
                }
            } else if lines.find("return").is_some() {
                let return_value = lines.replacen("return", "", 1);
                return Some(self.calculation(return_value));
            } else if lines.find("break").is_some() {
                return Some(f64::MAX);
            } else if lines == "exit" {
                exit(0);
            } else if lines == "" {
            } else {
                println!("コマンドが不正です: {}", lines)
            }
            remove_duplicates_variable(&mut self.memory);
            remove_duplicates_function(&mut self.name_space);
        }
        return None;
    }

    pub fn execute_block(&mut self, code: &String) -> Option<f64> {
        for lin in code.split("\n") {
            match self.execute(lin.to_string()) {
                Some(i) => return Some(i),
                None => {}
            }
        }
        return None;
    }

    fn reference_variable(&self, name: String) -> Option<usize> {
        let name = name.trim().replace(" ", "");
        match self
            .memory
            .iter()
            .position(|x| x.name == name.trim().replace(" ", ""))
        {
            Some(index) => Some(index),
            None => {
                println!("変数{name}が見つかりません");
                None
            }
        }
    }

    ///　関数の参照
    fn reference_function(&self, name: String) -> Option<usize> {
        let name = name.trim().replace(" ", "");
        match self
            .name_space
            .iter()
            .position(|x| x.name == name.trim().replace(" ", ""))
        {
            Some(index) => Some(index),
            None => {
                println!("関数{name}が見つかりません");
                None
            }
        }
    }

    /// 変数の参照(ログ出力なし)
    fn reference_variable_quiet(&self, name: String) -> Option<usize> {
        let name = name.trim().replace(" ", "");
        match self
            .memory
            .iter()
            .position(|x| x.name == name.trim().replace(" ", ""))
        {
            Some(index) => Some(index),
            None => None,
        }
    }

    /// 関数の参照(ログ出力なし)
    fn reference_function_quiet(&self, name: String) -> Option<usize> {
        let name = name.trim().replace(" ", "");
        match self
            .name_space
            .iter()
            .position(|x| x.name == name.trim().replace(" ", ""))
        {
            Some(index) => Some(index),
            None => None,
        }
    }

    fn calculation(&mut self, expr: String) -> f64 {
        let mut stack: Vec<f64> = Vec::new();
        let tokens = expr.split_whitespace();
        for i in tokens {
            let i = i.trim();
            if i.len() == 0 {
                continue;
            }
            match i.parse::<f64>() {
                Ok(num) => {
                    stack.push(num);
                    continue;
                }
                Err(_) => {
                    let y = stack.pop().unwrap_or(0.0);
                    let x = stack.pop().unwrap_or(0.0);
                    match i {
                        "+" => stack.push(x + y),
                        "-" => stack.push(x - y),
                        "*" => stack.push(x * y),
                        "/" => stack.push(x / y),
                        "%" => stack.push(x % y),
                        "^" => stack.push(x.powf(y)),
                        "=" => stack.push(if x == y { 1.0 } else { 0.0 }),
                        "&" => stack.push(if x != 0.0 && y != 0.0 { 1.0 } else { 0.0 }),
                        "|" => stack.push(if x != 0.0 || y != 0.0 { 1.0 } else { 0.0 }),
                        ">" => stack.push(if x > y { 1.0 } else { 0.0 }),
                        "<" => stack.push(if x < y { 1.0 } else { 0.0 }),
                        "!" => {
                            stack.push(x);
                            stack.push(if y == 0.0 { 1.0 } else { 0.0 })
                        }
                        _ => {
                            stack.push(x);
                            stack.push(y);

                            match self.reference_variable_quiet(i.to_string()) {
                                Some(i) => {
                                    stack.push(self.memory[i].value);
                                }
                                None => {}
                            }
                            match self.reference_function_quiet(i.to_string()) {
                                Some(i) => stack.push(
                                    match  Executor::new(&self.memory, &self.name_space).execute_block(&self.name_space[i].code) {
                                        Some(i) => i,
                                        None => 0.0,
                                    },
                                ),
                                None => {}
                            }
                        }
                    }
                }
            }
        }
        let result = stack.pop().unwrap_or(0.0);
        return result;
    }

    fn _compute(&mut self, expr: &String) -> f64 {
        let mut stack: Vec<f64> = Vec::new();
        let tokens = expr.split_whitespace();
        println!("+-- 計算処理 --");
        for i in tokens {
            let i = i.trim();
            if i.len() == 0 {
                continue;
            }
            println!("| Stack: {:?}  <=  '{}'", stack, i);
            match i.parse::<f64>() {
                Ok(num) => {
                    stack.push(num);
                    continue;
                }
                Err(_) => {
                    let y = stack.pop().unwrap_or(0.0);
                    let x = stack.pop().unwrap_or(0.0);
                    match i {
                        "+" => stack.push(x + y),
                        "-" => stack.push(x - y),
                        "*" => stack.push(x * y),
                        "/" => stack.push(x / y),
                        "%" => stack.push(x % y),
                        "^" => stack.push(x.powf(y)),
                        "=" => stack.push(if x == y { 1.0 } else { 0.0 }),
                        "&" => stack.push(if x != 0.0 && y != 0.0 { 1.0 } else { 0.0 }),
                        "|" => stack.push(if x != 0.0 || y != 0.0 { 1.0 } else { 0.0 }),
                        ">" => stack.push(if x > y { 1.0 } else { 0.0 }),
                        "<" => stack.push(if x < y { 1.0 } else { 0.0 }),
                        "!" => {
                            stack.push(x);
                            stack.push(if y == 0.0 { 1.0 } else { 0.0 })
                        }
                        _ => {
                            stack.push(x);
                            stack.push(y);

                            match self.reference_variable_quiet(i.to_string()) {
                                Some(i) => {
                                    stack.push(self.memory[i].value);
                                }
                                None => {}
                            }
                            match self.reference_function_quiet(i.to_string()) {
                                Some(index) => stack.push({
                                    println!("関数{i}を呼び出します");
                                    match  Executor::new(&self.memory, &self.name_space).execute_block(&self.name_space[index].code) {
                                        Some(indes) => indes,
                                        None => 0.0,
                                    }
                                }),
                                None => {}
                            }
                        }
                    }
                }
            };
        }
        let result = stack.pop().unwrap_or(0.0);
        println!("結果 = {}", result);
        return result;
    }
}
