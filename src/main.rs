use std::env;
use std::fs::File;
use std::io::{Error, Read};
mod executor;

#[cfg(test)]
mod tests; //テストモジュールを読み込む

// /// 変数のデータ
// #[derive(Clone)]
// struct Variable {
//     name: String,
//     types: String, // データ型
//     value: Value,
// }

// #[derive(Clone)]
// enum Value {
//     Number(Number),
//     Chars(Chars),
// }

// impl Variable {
//     fn get_number(&mut self) -> Number {
//         match &self.value {
//             Value::Number(number) => Number {
//                 expr: number.expr.clone(),
//                 value: number.value,
//             },
//             _ => Number {
//                 expr: "".to_string(),
//                 value: 0.0,
//             }, // デフォルト値
//         }
//     }

//     fn new_string(name: &str, value: &str) -> Self {
//         Self {
//             name: name.trim().replace(" ", "").to_string(),
//             types: "string".to_string(),
//             value: Value::Chars(Chars {
//                 value: value.replace("'", "").replace("\"", "").to_string(),
//             }),
//         }
//     }

//     fn new_number(name: &str, expr: String, value: f64) -> Self {
//         Self {
//             name: name.trim().replace(" ", "").to_string(),
//             types: "number".to_string(),
//             value: Value::Number(Number {
//                 expr: expr,
//                 value: value,
//             }),
//         }
//     }

//     fn get_string(&mut self) -> Chars {
//         match &self.value {
//             Value::Chars(value) => Chars {
//                 value: value.value.clone(),
//             },
//             _ => Chars {
//                 value: "".to_string(),
//             }, // デフォルト値
//         }
//     }
// }

// /// 数値(number)型
// #[derive(Clone)]
// struct Number {
//     value: f64,
//     expr: String,
// }

// /// 文字列(string)型
// #[derive(Clone)]
// struct Chars {
//     value: String,
// }

// /// 関数のデータ
// struct Func {
//     name: String,
//     code: String,
// }

// /// 変数の重複を削除する
// fn remove_duplicates_variable<'a>(
//     memory: &mut Vec<&mut Variable>,
// ) -> &'a mut Vec<&'a mut Variable> {
//     let mut seen_names = std::collections::HashMap::new();
//     let mut to_remove = Vec::new();

//     for (index, memory) in memory.iter().enumerate() {
//         if let Some(existing_index) = seen_names.get(&memory.name) {
//             to_remove.push(if existing_index < &index {
//                 *existing_index
//             } else {
//                 index
//             });
//         } else {
//             seen_names.insert(&memory.name, index);
//         }
//     }

//     to_remove.sort(); // Sort indices in ascending order

//     for (i, index) in to_remove.iter().enumerate() {
//         memory.remove(index - i); // Adjust for removed items before
//     }
//     return memory;
// }

// /// 関数の重複を削除する
// fn remove_duplicates_function(memory: &mut Vec<Func>) -> &mut Vec<Func> {
//     let mut seen_names = std::collections::HashMap::new();
//     let mut to_remove = Vec::new();

//     for (index, memory) in memory.iter().enumerate() {
//         if let Some(existing_index) = seen_names.get(&memory.name) {
//             to_remove.push(if existing_index < &index {
//                 *existing_index
//             } else {
//                 index
//             });
//         } else {
//             seen_names.insert(&memory.name, index);
//         }
//     }

//     to_remove.sort(); // Sort indices in ascending order

//     for (i, index) in to_remove.iter().enumerate() {
//         memory.remove(index - i); // Adjust for removed items before
//     }
//     return memory;
// }

// /// REPLで対話的に実行する
// fn interactive<'a>(memory: &'a mut Vec<&'a mut Variable>, name_space: &mut Vec<Func>) {
//     loop {
//         // 無限ループで「exit」コマンドまで永遠に実行
//         let mut lines = input::input("プログラム>>> ");
//         // 変数宣言
//         if lines.find("var").is_some() {
//             lines = lines.replacen("var", "", 1);
//             if lines.find("'").is_some() && lines.find("\"").is_some() {
//                 let params: Vec<&str> = lines.split("=").collect();
//                 let value = &params[1..].join("=").to_string();
//                 let mut variable = Variable::new_string(params[0], value);
//                 memory.push(&mut variable);
//             } else {
//                 let params: Vec<&str> = lines.split("=").collect();
//                 let value = compute(&params[1..].join("=").to_string(), memory, name_space);
//                 let mut variable = Variable::new_number(params[0], params[1..].join("="), value);
//                 memory.push(&mut variable);
//             }
//         //変数の式の再計算
//         } else if lines.find("calc").is_some() {
//             let name = lines.replacen("calc", "", 1);
//             match reference_variable(name, memory) {
//                 Some(index) => {
//                     let value = compute(
//                         &memory[index].to_owned().get_number().expr,
//                         memory,
//                         name_space,
//                     );
//                     memory[index].get_number().value = value;
//                     println!("再計算を実行しました");
//                 }
//                 None => {}
//             }
//         // メモリのデータの表示
//         } else if lines.find("mem").is_some() {
//             if memory.len() != 0 {
//                 println!("+-- メモリ内の変数 --");
//                 for i in memory.iter() {
//                     if i.types == "string" {
//                         println!(
//                             "| name: '{}' - type: {}型 -  - value: [{}]",
//                             i.name,
//                             i.types,
//                             i.get_string().value
//                         )
//                     } else {
//                         println!(
//                             "| name: '{}' - type: {}型 - expr: [{}] - value: {}",
//                             i.name,
//                             i.types,
//                             i.get_number().expr,
//                             i.get_number().value
//                         )
//                     }
//                 }
//             } else {
//                 println!("変数がありません");
//             }
//             if name_space.len() != 0 {
//                 println!("+-- メモリ内の関数 --");
//                 for i in name_space.iter() {
//                     println!("|+-- name: '{}' - len: {}", i.name, i.code.len());
//                     let mut number = 0; //行数
//                     for j in i.code.split('\n') {
//                         if j != "" {
//                             number += 1;
//                             println!("|| [{number}]: {j}");
//                         }
//                     }
//                 }
//             } else {
//                 println!("関数がありません");
//             }
//         //forループ
//         } else if lines.find("for").is_some() {
//             // ループ回数は式で計算する
//             println!("ループ回数を計算します");
//             let index: i32 =
//                 compute(&lines.replacen("for", "", 1), memory, name_space).round() as i32;
//             // 繰り返す関数
//             let mut stmt = String::new();
//             loop {
//                 let lines = input::input("forループ>>> ");
//                 if lines == "end for".to_string() {
//                     break; //「end for」までループ
//                 }
//                 stmt += &lines;
//                 stmt += "\n";
//             }

//             for i in 0..index {
//                 println!("{}回目のループ", i + 1); //ループ実行
//                 let status = execute(stmt.clone(), memory, name_space);
//                 match status {
//                     Some(i) => {
//                         if i == f64::MAX {
//                             //状態が1(break)の時はループを抜け出す
//                             break;
//                         }
//                     }
//                     None => {}
//                 }
//             }
//         // 関数(コマンドの集合体)の定義
//         } else if lines.find("func").is_some() {
//             let name = lines.trim().replacen("func", "", 1).replace(" ", "");
//             let mut stmt = String::new();
//             loop {
//                 let lines = input::input("funcブロック>>> ");
//                 if lines == "end func".to_string() {
//                     break;
//                 }
//                 stmt += &String::from("\n");
//                 stmt += &lines;
//             }
//             name_space.push(Func {
//                 name: name.trim().to_string(),
//                 code: stmt,
//             });
//         // 関数の呼び出し
//         } else if lines.find("call").is_some() {
//             lines = lines.replacen("call", "", 1);
//             let name = lines.trim().to_string();
//             match reference_function(name.clone(), name_space) {
//                 Some(index) => {
//                     println!("関数{name}を呼び出します");
//                     execute(name_space[index].code.clone(), memory, name_space);
//                 }
//                 None => {}
//             }
//         // if文
//         } else if lines.find("if").is_some() {
//             lines = lines.replacen("if", "", 1);
//             let mut stmt = String::new();
//             let mut else_code = String::new();
//             let expr = lines;
//             'a: /* 脱出用 */ loop {
//                 let lines = input::input("ifブロック>>> ");
//                 if lines == "else" { //「else」からelseブロック
//                     loop {
//                         let lines = input::input("elseブロック>>> ");
//                         if lines == "end if".to_string() {
//                             break 'a; //「end if」でif文終わり
//                         }
//                         else_code += &String::from("\n");
//                         else_code += &lines;
//                     }
//                 }
//                 if lines == "end if".to_string() {
//                     break 'a;
//                 }
//                 stmt += &String::from("\n");
//                 stmt += &lines;
//             }
//             println!("ifの条件式を評価します");
//             if compute(&expr, memory, name_space) != 0.0 {
//                 println!("条件が一致したので、実行します");
//                 execute(stmt, memory, name_space);
//             } else {
//                 if else_code != "" {
//                     // elseブロックがあるか?
//                     println!("条件が一致しなかったので、elseのコードを実行します");
//                     execute(else_code, memory, name_space);
//                 } else {
//                     println!("条件が一致しなかったので、実行しません");
//                 }
//             }
//         // whileループ
//         } else if lines.find("while").is_some() {
//             let mut stmt = String::new(); //条件式
//             let expr = lines.replacen("while", "", 1);
//             loop {
//                 let lines = input::input("whileループ>>> ");
//                 if lines == "end while".to_string() {
//                     break;
//                 }
//                 stmt += &lines;
//                 stmt += "\n";
//             }
//             loop {
//                 println!("whileの条件式を評価します");
//                 if compute(&expr.to_string(), memory, name_space) == 0.0 {
//                     println!("条件が一致しなかったので、ループを脱出します");
//                     break;
//                 } else {
//                     println!("条件が一致したので、ループを継続します");
//                     execute(stmt.clone(), memory, name_space);
//                 }
//             }
//         // input文(標準入力)
//         } else if lines.find("input").is_some() {
//             let name = lines.replacen("input", "", 1);

//             let inputed = input::input("[入力]> ");
//             let value = compute(&inputed, memory, name_space);
//             if inputed.find("'").is_some() && inputed.find("\"").is_some() {
//                 memory.push(&mut Variable {
//                     name: name,
//                     types: "string".to_string(),
//                     value: Value::Chars(Chars {
//                         value: inputed.replace("'", "").replace("\"", ""),
//                     }),
//                 });
//             } else {
//                 let value = compute(&inputed, memory, name_space);
//                 memory.push(&mut Variable {
//                     name: name,
//                     types: "number".to_string(),
//                     value: Value::Number(Number {
//                         value: value,
//                         expr: inputed,
//                     }),
//                 });
//             }
//         // コメント
//         } else if lines.find("#").is_some() {
//             // print文(標準出力)
//         } else if lines.find("print").is_some() {
//             lines = lines.replacen("print", "", 1);
//             let mut text: String = String::new();
//             let params = lines;
//             for i in params.split(",").collect::<Vec<&str>>() {
//                 if i.find("'").is_some() || i.find("\"").is_some() {
//                     //文字列か？
//                     text += &i.replace("'", "").replace("\"", "");
//                 } else {
//                     //文字列以外は式として扱われる
//                     text += &compute(&i.trim().to_string(), memory, name_space).to_string();
//                 }
//             }
//             println!("[出力]: {text}");
//         // delコマンド(変数の削除)
//         } else if lines.find("del").is_some() {
//             let name = lines.replacen("del", "", 1);
//             // 変数の参照
//             match reference_variable(name.clone(), memory) {
//                 Some(index) => {
//                     memory.remove(index);
//                     println!("変数{}を削除しました", name);
//                     continue;
//                 }
//                 None => {}
//             }
//             match reference_function(name.clone(), name_space) {
//                 Some(index) => {
//                     memory.remove(index);
//                     println!("関数{}を削除しました", name);
//                     continue;
//                 }
//                 None => {}
//             }
//         } else if lines.find("rand").is_some() {
//             lines = lines.replacen("rand", "", 1);
//             let params = lines.split(",").collect::<Vec<&str>>();
//             if params.len() < 3 {
//                 let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
//                 let temp: i64 = rng.gen_range(1, 10);
//                 memory.push(&mut Variable {
//                     name: params[0].trim().replace(" ", ""),
//                     types: "number".to_string(),
//                     value: Value::Number(Number {
//                         value: temp as f64,
//                         expr: temp.to_string(),
//                     }),
//                 });
//             } else {
//                 let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
//                 let temp: i64 = rng.gen_range(
//                     {
//                         println!("最小値を求めています");
//                         compute(&String::from(params[1]), memory, name_space).round() as i64
//                     },
//                     {
//                         println!("最大値を求めています");
//                         compute(&String::from(params[2]), memory, name_space).round() as i64
//                     },
//                 );
//                 memory.push(&mut Variable {
//                     name: params[0].trim().replace(" ", ""),
//                     types: "number".to_string(),
//                     value: Value::Number(Number {
//                         value: temp as f64,
//                         expr: temp.to_string(),
//                     }),
//                 });
//             }
//             println!("乱数を生成しました");
//         } else if lines == "exit" {
//             println!("終了します");
//             exit(0);
//         } else if lines == "" {
//         } else {
//             println!("コマンドが不正です: {}", lines)
//         }
//         remove_duplicates_variable(memory);
//         remove_duplicates_function(name_space);
//     }
// }

// ///　コードを一括実行する
// fn execute(
//     code: String,
//     memory: &mut Vec<&mut Variable>,
//     name_space: &mut Vec<Func>,
// ) -> Option<f64> {
//     let mut stmt = String::new(); // ブロックのステートメント
//     let mut else_stmt = String::new(); // elseステートメント
//     let mut count = 0; // ループカウンタ
//     let mut name = String::new(); // 関数の名前
//     let mut expr = String::new(); // 条件式
//     let mut mode = String::from("normal"); // 制御ブロックの状態
//     let mut old_mode = String::new(); // 元のモード
//     let mut number = 0; //実行している行

//     // 改行区切りで実行する
//     for mut lines in code.split("\n") {
//         lines = lines.trim_start().trim_end();
//         if lines == "" {
//             continue;
//         } // 空白の行を飛ばす
//         number += 1; // 進捗メッセージ
//                      // forモードの場合
//         if mode == "for".to_string() {
//             if lines == "end for" {
//                 // 「end for」でループ終わり
//                 for i in 0..count {
//                     println!("{}回目のループ", i + 1); //ループ実行
//                     let status = execute(stmt.clone(), memory, name_space);
//                     match status {
//                         Some(i) => {
//                             if i == f64::MAX {
//                                 //状態が1(break)の時はループを抜け出す
//                                 break;
//                             } else {
//                                 return Some(i);
//                             }
//                         }
//                         None => {}
//                     }
//                 }
//             } else {
//                 stmt += lines;
//                 stmt += "\n";
//             }
//         } else if mode == "if".to_string() {
//             if lines == "else" {
//                 old_mode = mode;
//                 mode = "else".to_string()
//             } else if lines == "end if" {
//                 println!("ifの条件式を評価します");
//                 if compute(&expr, memory, name_space) != 0.0 {
//                     println!("条件が一致したので、実行します");
//                     let status = execute(stmt.clone(), memory, name_space);
//                     match status {
//                         Some(i) => {
//                             if i == f64::MAX {
//                                 //状態が1(break)の時はループを抜け出す
//                                 break;
//                             } else {
//                                 return Some(i);
//                             }
//                         }
//                         None => {}
//                     }
//                     stmt = String::new();
//                 } else {
//                     println!("条件が一致しなかったので、実行しません");
//                     stmt = String::new();
//                 }
//                 mode = old_mode.clone();
//             } else {
//                 stmt += lines;
//                 stmt += "\n";
//             }
//         } else if mode == "func".to_string() {
//             if lines == "end func" {
//                 name_space.push(Func {
//                     name: name.clone(),
//                     code: stmt.clone(),
//                 });
//                 stmt = String::new();
//                 mode = old_mode.clone();
//             } else {
//                 stmt += lines;
//                 stmt += &String::from("\n");
//             }
//         } else if mode == "else".to_string() {
//             if lines == "end if" {
//                 println!("ifの条件式を評価します");
//                 if compute(&expr, memory, name_space) == 0.0 {
//                     println!("条件が一致しなかったので、elseのコードを実行します");
//                     let status = execute(else_stmt.clone(), memory, name_space);
//                     match status {
//                         Some(i) => {
//                             if i == f64::MAX {
//                                 //状態が1(break)の時はループを抜け出す
//                                 break;
//                             } else {
//                                 return Some(i);
//                             }
//                         }
//                         None => {}
//                     }
//                     else_stmt = String::new();
//                     stmt = String::new();
//                 } else {
//                     println!("条件が一致したので、実行します");

//                     let status = execute(stmt.clone(), memory, name_space);
//                     match status {
//                         Some(i) => {
//                             if i == f64::MAX {
//                                 //状態が1(break)の時はループを抜け出す
//                                 break;
//                             } else {
//                                 return Some(i);
//                             }
//                         }
//                         None => {}
//                     }
//                     else_stmt = String::new();
//                     stmt = String::new();
//                 }
//                 mode = old_mode.clone();
//             } else {
//                 else_stmt += lines;
//                 else_stmt += &String::from("\n");
//             }
//         } else if mode == "while".to_string() {
//             if lines == "end while" {
//                 loop {
//                     println!("whileの条件式を評価します");
//                     if compute(&expr, memory, name_space) == 0.0 {
//                         println!("条件が一致しなかったので、ループを脱出します");
//                         stmt = String::new();
//                         break;
//                     }
//                     let status = execute(stmt.clone(), memory, name_space);
//                     match status {
//                         Some(i) => {
//                             if i == f64::MAX {
//                                 //状態が1(break)の時はループを抜け出す
//                                 break;
//                             } else {
//                                 return Some(i);
//                             }
//                         }
//                         None => {}
//                     }
//                 }
//                 mode = old_mode.clone();
//             } else {
//                 stmt += lines;
//                 stmt += "\n";
//             }
//         } else {
//             println!("-- {number}行: [{lines}]を実行");
//             if lines.find("var").is_some() {
//                 let new_lines = lines.replacen("var", "", 1);
//                 lines = &new_lines;
//                 if lines.find("'").is_some() && lines.find("\"").is_some() {
//                     let params: Vec<&str> = lines.split("=").collect();
//                     let value = &params[1..].join("=").to_string();
//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "string".to_string(),
//                         value: Value::Chars(Chars {
//                             value: value.replace("'", "").replace("\"", ""),
//                         }),
//                     });
//                 } else {
//                     let params: Vec<&str> = lines.split("=").collect();
//                     let value = compute(&params[1..].join("=").to_string(), memory, name_space);
//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value: value,
//                             expr: params[1..].join("=").to_string(),
//                         }),
//                     });
//                 }
//             } else if lines.find("calc").is_some() {
//                 let new_lines = lines.replacen("calc", "", 1);
//                 let name = &new_lines;
//                 match reference_variable(name.to_owned(), memory) {
//                     Some(index) => {
//                         let value = compute(
//                             &memory[index].to_owned().get_number().expr,
//                             memory,
//                             name_space,
//                         );
//                         memory[index].get_number().value = value;
//                         println!("再計算を実行しました");
//                     }
//                     None => {}
//                 }
//             } else if lines.find("mem").is_some() {
//                 if memory.len() != 0 {
//                     println!("+-- メモリ内の変数 --");
//                     for mut i in memory.iter() {
//                         if i.types == "string" {
//                             println!(
//                                 "| name: '{}' - type: {}型 -  - value: [{}]",
//                                 i.name,
//                                 i.types,
//                                 i.get_string().value
//                             )
//                         } else {
//                             println!(
//                                 "| name: '{}' - type: {}型 - expr: [{}] - value: {}",
//                                 i.name,
//                                 i.types,
//                                 i.get_number().expr,
//                                 i.get_number().value
//                             )
//                         }
//                     }
//                 } else {
//                     println!("変数がありません");
//                 }
//                 if name_space.len() != 0 {
//                     println!("+-- メモリ内の関数 --");
//                     for i in name_space.iter() {
//                         println!("|+-- name: '{}' - len: {}", i.name, i.code.len());
//                         let mut number = 0; //行数
//                         for j in i.code.split('\n') {
//                             if j != "" {
//                                 number += 1;
//                                 println!("|| [{number}]: {j}");
//                             }
//                         }
//                     }
//                 } else {
//                     println!("関数がありません");
//                 }
//             } else if lines.find("func").is_some() {
//                 let new_lines = lines.replacen("func", "", 1);
//                 name = new_lines;
//                 mode = "func".to_string();
//             } else if lines.find("call").is_some() {
//                 let new_lines = lines.replacen("call", "", 1);
//                 let name = &new_lines;
//                 let codes = match reference_function(name.to_owned(), name_space) {
//                     Some(index) => name_space[index].code.clone(),
//                     None => {
//                         println!("関数{name}が見つかりません");
//                         "".to_string();
//                         continue;
//                     }
//                 };
//                 println!("関数{name}を呼び出します");
//                 execute(codes.clone(), memory, name_space);
//             } else if lines.find("for").is_some() {
//                 let new_lines = lines.replacen("for", "", 1);
//                 count = compute(&new_lines, memory, name_space) as i32;
//                 old_mode = mode;
//                 mode = "for".to_string();
//             } else if lines.find("if").is_some() {
//                 let new_lines = lines.replacen("if", "", 1);
//                 expr = new_lines;
//                 old_mode = mode;
//                 mode = "if".to_string()
//             } else if lines.find("while").is_some() {
//                 let new_lines = lines.replacen("while", "", 1);
//                 expr = new_lines;
//                 old_mode = mode;
//                 mode = "while".to_string();
//             } else if lines.find("input").is_some() {
//                 let new_lines = lines.replacen("input", "", 1);
//                 let name = &new_lines;

//                 let inputed = input::input("[入力]> ");
//                 let value = compute(&inputed, memory, name_space);
//                 if inputed.find("'").is_some() && inputed.find("\"").is_some() {
//                     memory.push(&mut Variable {
//                         name: name.to_owned(),
//                         types: "string".to_string(),
//                         value: Value::Chars(Chars {
//                             value: inputed.replace("'", "").replace("\"", ""),
//                         }),
//                     });
//                 } else {
//                     let value = compute(&inputed, memory, name_space);
//                     memory.push(&mut Variable {
//                         name: name.to_owned(),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value: value,
//                             expr: inputed,
//                         }),
//                     });
//                 }
//             } else if lines.find("print").is_some() {
//                 let new_lines = lines.replacen("print", "", 1);
//                 let mut text = String::new();
//                 let params = &new_lines;
//                 for i in params.split(",").collect::<Vec<&str>>() {
//                     if i.find("'").is_some() || i.find("\"").is_some() {
//                         //文字列か？
//                         text += &i.replace("'", "").replace("\"", "");
//                     } else {
//                         //文字列以外は式として扱われる
//                         text += &compute(&i.trim().to_string(), memory, name_space).to_string();
//                     }
//                 }
//                 println!("[出力]: {text}");
//             } else if lines.find("rand").is_some() {
//                 let new_lines = lines.replacen("rand", "", 1);
//                 let params = new_lines.split(",").collect::<Vec<&str>>();
//                 if params.len() < 3 {
//                     let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
//                     let temp: i64 = rng.gen_range(1, 10);
//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value: temp as f64,
//                             expr: temp.to_string(),
//                         }),
//                     });
//                 } else {
//                     let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
//                     let temp: i64 = rng.gen_range(
//                         {
//                             println!("最小値を求めています");
//                             compute(&String::from(params[1]), memory, name_space).round() as i64
//                         },
//                         {
//                             println!("最大値を求めています");
//                             compute(&String::from(params[2]), memory, name_space).round() as i64
//                         },
//                     );
//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value: temp as f64,
//                             expr: temp.to_string(),
//                         }),
//                     });
//                 }
//                 println!("乱数を生成しました");
//             } else if lines.find("del").is_some() {
//                 let new_lines = lines.replacen("del", "", 1);
//                 let name = &new_lines;
//                 match reference_variable(name.clone(), memory) {
//                     Some(index) => {
//                         memory.remove(index);
//                         println!("変数{}を削除しました", name);
//                         continue;
//                     }
//                     None => {}
//                 }
//                 match reference_function(name.to_owned(), name_space) {
//                     Some(index) => {
//                         memory.remove(index);
//                         println!("関数{}を削除しました", name);
//                         continue;
//                     }
//                     None => {}
//                 }
//             } else if lines.find("return").is_some() {
//                 let return_value = lines.replacen("return", "", 1);
//                 return Some(compute(&return_value, memory, name_space));
//             } else if lines.find("break").is_some() {
//                 return Some(f64::MAX);
//             } else if lines.find("#").is_some() {
//             } else if lines == "exit" {
//                 println!("終了します");
//                 exit(0);
//             } else if lines == "" {
//             } else {
//                 println!("コマンドが不正です: {}", lines)
//             }
//             remove_duplicates_variable(memory);
//             remove_duplicates_function(name_space);
//         }
//     }
//     return None;
// }

// /// コードを実行する
// fn script(
//     code: String,
//     memory: &mut Vec<&mut Variable>,
//     name_space: &mut Vec<Func>,
// ) -> Option<f64> {
//     let mut stmt = String::new(); // ブロックのステートメント
//     let mut else_stmt = String::new(); // elseステートメント
//     let mut count = 0; // ループカウンタ
//     let mut name = String::new(); // 関数の名前
//     let mut expr = String::new(); // 条件式
//     let mut flag: bool = false; // 条件式の結果
//     let mut mode = String::from("normal"); // 制御ブロックの状態
//     let mut old_mode = String::new(); // 元のモード
//     let mut nest_if = 0; // ifネストの階層を表す
//     let mut nest_for = 0; // forネストの階層を表す
//     let mut nest_while = 0; // whileネストの階層を表す
//     let mut nest_func = 0; // funcネストの階層を表す

//     for mut lines in code.split("\n") {
//         lines = lines.trim_start().trim_end();
//         if mode == "for".to_string() {
//             if lines.find("end for").is_some() {
//                 if nest_for > 0 {
//                     nest_for -= 1;
//                     stmt += lines;
//                     stmt += "\n";
//                 } else {
//                     for _ in 0..count {
//                         let status = script(stmt.clone(), memory, name_space);
//                         match status {
//                             Some(i) => {
//                                 if i == f64::MAX {
//                                     //状態が1(break)の時はループを抜け出す
//                                     break;
//                                 } else {
//                                     return Some(i);
//                                 }
//                             }
//                             None => {}
//                         }
//                     }
//                     stmt = String::new();
//                     mode = old_mode.clone();
//                 }
//             } else if lines.find("for").is_some() {
//                 nest_for += 1;
//                 stmt += lines;
//                 stmt += "\n";
//             } else {
//                 stmt += lines;
//                 stmt += "\n";
//             }
//         } else if mode == "if".to_string() {
//             if lines.find("else").is_some() {
//                 old_mode = mode;
//                 mode = "else".to_string()
//             } else if lines.find("end if").is_some() {
//                 if nest_if > 0 {
//                     nest_if -= 1;
//                     stmt += lines;
//                     stmt += "\n";
//                 } else {
//                     if flag {
//                         let status = script(stmt.clone(), memory, name_space);
//                         match status {
//                             Some(i) => {
//                                 if i == f64::MAX {
//                                     return Some(f64::MAX);
//                                 } else {
//                                     return Some(i);
//                                 }
//                             }
//                             None => {}
//                         }
//                         stmt = String::new();
//                     } else {
//                         stmt = String::new();
//                     }
//                     mode = old_mode.clone();
//                 }
//             } else if lines.find("if").is_some() {
//                 nest_if += 1;
//                 stmt += lines;
//                 stmt += "\n";
//             } else {
//                 stmt += lines;
//                 stmt += "\n";
//             }
//         } else if mode == "func".to_string() {
//             if lines.find("end func").is_some() {
//                 if nest_func > 0 {
//                     nest_func -= 1;
//                 } else {
//                     name_space.push(Func {
//                         name: name.clone(),
//                         code: stmt.clone(),
//                     });
//                     stmt = String::new();
//                     mode = old_mode.clone();
//                 }
//             } else if lines.find("func").is_some() {
//                 nest_func += 1;
//             } else {
//                 stmt += lines;
//                 stmt += &String::from("\n");
//             }
//         } else if mode == "else".to_string() {
//             if lines.find("end if").is_some() {
//                 if nest_if > 0 {
//                     nest_if -= 1;
//                     stmt += lines;
//                     stmt += "\n";
//                 } else {
//                     if !flag {
//                         let status = script(else_stmt.clone(), memory, name_space);
//                         match status {
//                             Some(i) => {
//                                 if i == f64::MAX {
//                                     return Some(f64::MAX);
//                                 } else {
//                                     return Some(i);
//                                 }
//                             }
//                             None => {}
//                         }
//                         else_stmt = String::new();
//                         stmt = String::new();
//                     } else {
//                         let status = script(stmt.clone(), memory, name_space);
//                         match status {
//                             Some(i) => {
//                                 if i == f64::MAX {
//                                     return Some(f64::MAX);
//                                 } else {
//                                     return Some(i);
//                                 }
//                             }
//                             None => {}
//                         }
//                         else_stmt = String::new();
//                         stmt = String::new();
//                     }
//                     mode = old_mode.clone();
//                 }
//             } else if lines.find("if").is_some() {
//                 nest_if += 1;
//                 stmt += lines;
//                 stmt += "\n";
//                 mode = "if".to_string();
//             } else {
//                 else_stmt += lines;
//                 else_stmt += &String::from("\n");
//             }
//         } else if mode == "while".to_string() {
//             if lines.find("end while").is_some() {
//                 if nest_while > 0 {
//                     nest_while -= 1;
//                 } else {
//                     loop {
//                         if calculation(&expr, memory, name_space) == 0.0 {
//                             stmt = String::new();
//                             break;
//                         } else {
//                             let status = script(stmt.clone(), memory, name_space);
//                             match status {
//                                 Some(i) => {
//                                     if i == f64::MAX {
//                                         //状態が1(break)の時はループを抜け出す
//                                         break;
//                                     } else {
//                                         return Some(i);
//                                     }
//                                 }
//                                 None => {}
//                             }
//                         }
//                     }
//                     mode = old_mode.clone();
//                 }
//             } else if lines.find("while").is_some() {
//                 nest_while += 1;
//             } else {
//                 stmt += lines;
//                 stmt += "\n";
//             }
//         } else {
//             if lines.find("var").is_some() {
//                 let new_lines = lines.replacen("var", "", 1);
//                 lines = &new_lines;
//                 let params: Vec<&str> = lines.split("=").collect();
//                 let value = calculation(&params[1..].join("=").to_string(), memory, name_space);
//                 if lines.find("'").is_some() && lines.find("\"").is_some() {
//                     let params: Vec<&str> = lines.split("=").collect();
//                     let value = &params[1..].join("=").to_string();
//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "string".to_string(),
//                         value: Value::Chars(Chars {
//                             value: value.replace("'", "").replace("\"", ""),
//                         }),
//                     });
//                 } else {
//                     let params: Vec<&str> = lines.split("=").collect();
//                     let value = compute(&params[1..].join("=").to_string(), memory, name_space);
//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value: value,
//                             expr: params[1..].join("=").to_string(),
//                         }),
//                     });
//                 }
//             } else if lines.find("calc").is_some() {
//                 let new_lines = lines.replacen("calc", "", 1);
//                 let name = &new_lines;
//                 match reference_variable(name.to_string(), memory) {
//                     Some(index) => {
//                         let value = calculation(
//                             &memory[index].to_owned().get_number().expr,
//                             memory,
//                             name_space,
//                         );
//                         memory[index].get_number().value = value;
//                     }
//                     None => {}
//                 }
//             } else if lines.find("func").is_some() {
//                 let new_lines = lines.trim().replacen("func", "", 1).replace(" ", "");
//                 name = new_lines.replace(" ", "");
//                 mode = "func".to_string();
//             } else if lines.find("call").is_some() {
//                 let new_lines = lines.replacen("call", "", 1);
//                 let name = &new_lines.replace(" ", "");
//                 match reference_function(name.clone(), name_space) {
//                     Some(index) => {
//                         script(name_space[index].code.clone(), memory, name_space);
//                     }
//                     None => {}
//                 }
//             } else if lines.find("for").is_some() {
//                 let new_lines = lines.replacen("for", "", 1);
//                 count = calculation(&new_lines, memory, name_space) as i32;
//                 old_mode = mode;
//                 mode = "for".to_string();
//             } else if lines.find("if").is_some() {
//                 let new_lines = lines.replacen("if", "", 1);
//                 flag = calculation(&new_lines, memory, name_space) != 0.0;
//                 old_mode = mode;
//                 mode = "if".to_string()
//             } else if lines.find("while").is_some() {
//                 let new_lines = lines.replacen("while", "", 1);
//                 expr = new_lines;
//                 old_mode = mode;
//                 mode = "while".to_string();
//             } else if lines.find("input").is_some() {
//                 let new_lines = lines.replacen("input", "", 1);
//                 let name = &new_lines;

//                 let inputed = input::input("[入力]> ");
//                 let value = compute(&inputed, memory, name_space);
//                 if inputed.find("'").is_some() && inputed.find("\"").is_some() {
//                     memory.push(&mut Variable {
//                         name: name.to_owned(),
//                         types: "string".to_string(),
//                         value: Value::Chars(Chars {
//                             value: inputed.replace("'", "").replace("\"", ""),
//                         }),
//                     });
//                 } else {
//                     let value = compute(&inputed, memory, name_space);
//                     memory.push(&mut Variable {
//                         name: name.to_owned(),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value: value,
//                             expr: inputed,
//                         }),
//                     });
//                 }
//             } else if lines.find("print").is_some() {
//                 let new_lines = lines.replacen("print", "", 1);
//                 let mut text = String::new();
//                 let params = &new_lines;
//                 for i in params.split(",").collect::<Vec<&str>>() {
//                     if i.find("'").is_some() || i.find("\"").is_some() {
//                         //文字列か？
//                         text += &i.replace("'", "").replace("\"", "");
//                     } else {
//                         //文字列以外は式として扱われる
//                         text += &calculation(&i.trim().to_string(), memory, name_space).to_string();
//                     }
//                 }
//                 println!("{text}");
//             } else if lines.find("del").is_some() {
//                 let new_lines = lines.replacen("del", "", 1);
//                 let name = &new_lines;
//                 match reference_variable(name.clone(), memory) {
//                     Some(index) => {
//                         memory.remove(index);
//                         continue;
//                     }
//                     None => {}
//                 }
//                 match reference_function(name.to_owned(), name_space) {
//                     Some(index) => {
//                         memory.remove(index);
//                         continue;
//                     }
//                     None => {}
//                 }
//             } else if lines.find("#").is_some() {
//             } else if lines.find("rand").is_some() {
//                 let new_lines = lines.replacen("rand", "", 1);
//                 let params = new_lines.split(",").collect::<Vec<&str>>();
//                 if params.len() < 3 {
//                     let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
//                     let temp: i64 = rng.gen_range(1, 10);
//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value: temp as f64,
//                             expr: temp.to_string(),
//                         }),
//                     });
//                 } else {
//                     let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
//                     let temp: i64 = rng.gen_range(
//                         calculation(&String::from(params[1]), memory, name_space).round() as i64,
//                         calculation(&String::from(params[2]), memory, name_space).round() as i64,
//                     );
//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value: temp as f64,
//                             expr: temp.to_string(),
//                         }),
//                     });
//                 }
//             } else if lines.find("return").is_some() {
//                 let return_value = lines.replacen("return", "", 1);
//                 return Some(calculation(&return_value, memory, name_space));
//             } else if lines.find("break").is_some() {
//                 return Some(f64::MAX);
//             } else if lines == "exit" {
//                 exit(0);
//             } else if lines == "" {
//             } else {
//                 println!("コマンドが不正です: {}", lines)
//             }
//             remove_duplicates_variable(memory);
//             remove_duplicates_function(name_space);
//         }
//     }
//     return None;

//     fn calculation(
//         expr: &String,
//         memory: &mut Vec<&mut Variable>,
//         name_space: &mut Vec<Func>,
//     ) -> f64 {
//         let mut stack: Vec<f64> = Vec::new();
//         let tokens = expr.split_whitespace();
//         for i in tokens {
//             let i = i.trim();
//             if i.len() == 0 {
//                 continue;
//             }
//             match i.parse::<f64>() {
//                 Ok(num) => {
//                     stack.push(num);
//                     continue;
//                 }
//                 Err(_) => {
//                     let y = stack.pop().unwrap_or(0.0);
//                     let x = stack.pop().unwrap_or(0.0);
//                     match i {
//                         "+" => stack.push(x + y),
//                         "-" => stack.push(x - y),
//                         "*" => stack.push(x * y),
//                         "/" => stack.push(x / y),
//                         "%" => stack.push(x % y),
//                         "^" => stack.push(x.powf(y)),
//                         "=" => stack.push(if x == y { 1.0 } else { 0.0 }),
//                         "&" => stack.push(if x != 0.0 && y != 0.0 { 1.0 } else { 0.0 }),
//                         "|" => stack.push(if x != 0.0 || y != 0.0 { 1.0 } else { 0.0 }),
//                         ">" => stack.push(if x > y { 1.0 } else { 0.0 }),
//                         "<" => stack.push(if x < y { 1.0 } else { 0.0 }),
//                         "!" => {
//                             stack.push(x);
//                             stack.push(if y == 0.0 { 1.0 } else { 0.0 })
//                         }
//                         _ => {
//                             stack.push(x);
//                             stack.push(y);

//                             match reference_variable_quiet(i.to_string(), memory) {
//                                 Some(i) => {
//                                     stack.push(memory[i].get_number().value);
//                                 }
//                                 None => {}
//                             }
//                             match reference_function_quiet(i.to_string(), name_space) {
//                                 Some(i) => stack.push(
//                                     match script(name_space[i].code.clone(), memory, name_space) {
//                                         Some(i) => i,
//                                         None => 0.0,
//                                     },
//                                 ),
//                                 None => {}
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//         let result = stack.pop().unwrap_or(0.0);
//         return result;
//     }
// }

// /// ファイルをデバッグする
// fn debug(code: String, memory: &mut Vec<&mut Variable>, name_space: &mut Vec<Func>) -> Option<f64> {
//     let mut stmt = String::new(); // ブロックのステートメント
//     let mut else_stmt = String::new(); // elseステートメント
//     let mut count = 0; // ループカウンタ
//     let mut name = String::new(); // 関数の名前
//     let mut expr = String::new(); // 条件式
//     let mut mode = String::from("normal"); // 制御ブロックの状態
//     let mut old_mode = String::new(); // 元のモード
//     let mut number = 0; //実行している行
//     let mut nest_if = 0; // ifネストの階層を表す
//     let mut nest_for = 0; // forネストの階層を表す
//     let mut nest_while = 0; // whileネストの階層を表す
//     let mut nest_func = 0; // funcネストの階層を表す

//     // 改行区切りで実行する
//     for mut lines in code.split("\n") {
//         lines = lines.trim_start().trim_end();
//         if lines == "" {
//             continue;
//         } // 空白の行を飛ばす
//         number += 1; // 進捗メッセージ
//                      // forモードの場合
//         if mode == "for".to_string() {
//             if lines == "end for" {
//                 if nest_for > 0 {
//                     nest_for -= 1;
//                     stmt += lines;
//                     stmt += "\n";
//                 } else {
//                     // 「end for」でループ終わり
//                     for _ in 0..count {
//                         let status = debug(stmt.clone(), memory, name_space);
//                         match status {
//                             Some(i) => {
//                                 if i == f64::MAX {
//                                     //状態が1(break)の時はループを抜け出す
//                                     break;
//                                 } else {
//                                     return Some(i);
//                                 }
//                             }
//                             None => {}
//                         }
//                     }
//                 }
//                 stmt = String::new();
//                 mode = old_mode.clone();
//             } else if lines.find("for").is_some() {
//                 nest_for += 1;
//                 stmt += lines;
//                 stmt += "\n";
//             } else {
//                 stmt += lines;
//                 stmt += "\n";
//             }
//         } else if mode == "if".to_string() {
//             if lines == "else" {
//                 old_mode = mode;
//                 mode = "else".to_string()
//             } else if lines == "end if" {
//                 if nest_if > 0 {
//                     nest_if -= 1;
//                     stmt += lines;
//                     stmt += "\n";
//                 } else {
//                     println!("ifの条件式を評価します");
//                     if compute(&expr, memory, name_space) != 0.0 {
//                         println!("条件が一致したので、実行します");
//                         let status = debug(stmt.clone(), memory, name_space);
//                         match status {
//                             Some(i) => {
//                                 if i == f64::MAX {
//                                     //状態が1(break)の時はループを抜け出す
//                                     break;
//                                 } else {
//                                     return Some(i);
//                                 }
//                             }
//                             None => {}
//                         }
//                         stmt = String::new();
//                     } else {
//                         println!("条件が一致しなかったので、実行しません");
//                         stmt = String::new();
//                     }
//                 }
//                 mode = old_mode.clone();
//             } else if lines.find("if").is_some() {
//                 nest_if += 1;
//                 stmt += lines;
//                 stmt += "\n";
//             } else {
//                 stmt += lines;
//                 stmt += "\n";
//             }
//         } else if mode == "func".to_string() {
//             if lines == "end func" {
//                 if nest_func > 0 {
//                     nest_func -= 1;
//                     stmt += lines;
//                     stmt += "\n";
//                 } else {
//                     name_space.push(Func {
//                         name: name.clone(),
//                         code: stmt.clone(),
//                     });
//                     stmt = String::new();
//                     mode = old_mode.clone();
//                 }
//             } else if lines.find("func").is_some() {
//                 nest_func += 1;
//                 stmt += lines;
//                 stmt += "\n";
//             } else {
//                 stmt += lines;
//                 stmt += &String::from("\n");
//             }
//         } else if mode == "else".to_string() {
//             if lines == "end if" {
//                 if nest_if > 0 {
//                     nest_if -= 1;
//                     stmt += lines;
//                     stmt += "\n";
//                 } else {
//                     println!("ifの条件式を評価します");
//                     if compute(&expr, memory, name_space) == 0.0 {
//                         println!("条件が一致しなかったので、elseのコードを実行します");
//                         let status = debug(else_stmt.clone(), memory, name_space);
//                         match status {
//                             Some(i) => {
//                                 if i == f64::MAX {
//                                     //状態が1(break)の時はループを抜け出す
//                                     break;
//                                 } else {
//                                     return Some(i);
//                                 }
//                             }
//                             None => {}
//                         }
//                         else_stmt = String::new();
//                         stmt = String::new();
//                     } else {
//                         println!("条件が一致したので、実行します");
//                         let status = debug(stmt.clone(), memory, name_space);
//                         match status {
//                             Some(i) => {
//                                 if i == f64::MAX {
//                                     //状態が1(break)の時はループを抜け出す
//                                     break;
//                                 } else {
//                                     return Some(i);
//                                 }
//                             }
//                             None => {}
//                         }
//                         else_stmt = String::new();
//                         stmt = String::new();
//                     }
//                 }
//                 mode = old_mode.clone();
//             } else if lines.find("if").is_some() {
//                 nest_if += 1;
//                 stmt += lines;
//                 stmt += "\n";
//             } else {
//                 else_stmt += lines;
//                 else_stmt += &String::from("\n");
//             }
//         } else if mode == "while".to_string() {
//             if lines == "end while" {
//                 if nest_while > 0 {
//                     nest_while -= 1;
//                     stmt += lines;
//                     stmt += "\n";
//                 } else {
//                     loop {
//                         println!("whileの条件式を評価します");
//                         if compute(&expr, memory, name_space) == 0.0 {
//                             println!("条件が一致しなかったので、ループを脱出します");
//                             stmt = String::new();
//                             break;
//                         }
//                         let status = debug(stmt.clone(), memory, name_space);
//                         match status {
//                             Some(i) => {
//                                 if i == f64::MAX {
//                                     //状態が1(break)の時はループを抜け出す
//                                     break;
//                                 } else {
//                                     return Some(i);
//                                 }
//                             }
//                             None => {}
//                         }
//                     }
//                 }
//                 mode = old_mode.clone();
//             } else if lines.find("while").is_some() {
//                 nest_while += 1;
//                 stmt += lines;
//                 stmt += "\n";
//             } else {
//                 stmt += lines;
//                 stmt += "\n";
//             }
//         } else {
//             println!("+- {number}行: [{lines}]を実行");
//             if lines.find("var").is_some() {
//                 let new_lines = lines.replacen("var", "", 1);
//                 let params: Vec<&str> = new_lines.split("=").collect();
//                 let value = compute(&params[1..].join("=").to_string(), memory, name_space);
//                 if lines.find("'").is_some() && lines.find("\"").is_some() {
//                     let params: Vec<&str> = lines.split("=").collect();
//                     let value = &params[1..].join("=").to_string();
//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "string".to_string(),
//                         value: Value::Chars(Chars {
//                             value: value.replace("'", "").replace("\"", ""),
//                         }),
//                     });
//                 } else {
//                     let params: Vec<&str> = lines.split("=").collect();
//                     let value = compute(&params[1..].join("=").to_string(), memory, name_space);
//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value: value,
//                             expr: params[1..].join("=").to_string(),
//                         }),
//                     });
//                 }
//             } else if lines.find("calc").is_some() {
//                 let new_lines = lines.replacen("calc", "", 1);
//                 let name = &new_lines;
//                 match reference_variable(name.to_owned(), memory) {
//                     Some(index) => {
//                         let value = compute(
//                             &memory[index].to_owned().get_number().expr,
//                             memory,
//                             name_space,
//                         );
//                         memory[index].get_number().value = value;
//                         println!("再計算を実行しました");
//                     }
//                     None => {}
//                 }
//             } else if lines.find("mem").is_some() {
//                 println!("+-- メモリ内の変数 --");
//                 for i in memory.iter() {
//                     if i.types == "string" {
//                         println!(
//                             "| name: '{}' - type: {}型 -  - value: [{}]",
//                             i.name,
//                             i.types,
//                             i.get_string().value
//                         )
//                     } else {
//                         println!(
//                             "| name: '{}' - type: {}型 - expr: [{}] - value: {}",
//                             i.name,
//                             i.types,
//                             i.get_number().expr,
//                             i.get_number().value
//                         )
//                     }
//                 }
//                 println!("+-- メモリ内の関数 --");
//                 for i in name_space.iter() {
//                     println!("| name: '{}'", i.name);
//                     let mut number = 0; //行数
//                     for j in i.code.split('\n') {
//                         number += 1;
//                         println!("|| [{number}]: {j}");
//                     }
//                 }
//             } else if lines.find("func").is_some() {
//                 name = lines.trim().replacen("func", "", 1).replace(" ", "");
//                 mode = "func".to_string();
//                 println!("関数{name}を定義します");
//             } else if lines.find("call").is_some() {
//                 name = lines.trim().replacen("call", "", 1).replace(" ", "");
//                 let codes = match name_space.iter().position(|x| x.name == name.to_string()) {
//                     Some(index) => name_space[index].code.clone(),
//                     None => {
//                         println!("関数{name}が見つかりません");
//                         "".to_string();
//                         continue;
//                     }
//                 };
//                 println!("関数{name}を呼び出します");
//                 debug(codes.clone(), memory, name_space);
//             } else if lines.find("for").is_some() {
//                 let new_lines = lines.replacen("for", "", 1);
//                 count = compute(&new_lines, memory, name_space) as i32;
//                 old_mode = mode;
//                 mode = "for".to_string();
//             } else if lines.find("if").is_some() {
//                 let new_lines = lines.replacen("if", "", 1);
//                 expr = new_lines;
//                 old_mode = mode;
//                 mode = "if".to_string()
//             } else if lines.find("while").is_some() {
//                 let new_lines = lines.replacen("while", "", 1);
//                 expr = new_lines;
//                 old_mode = mode;
//                 mode = "while".to_string();
//             } else if lines.find("input").is_some() {
//                 let new_lines = lines.replacen("input", "", 1);
//                 let name = &new_lines;

//                 let inputed = input::input("[入力]> ");
//                 let value = compute(&inputed, memory, name_space);
//                 if inputed.find("'").is_some() && inputed.find("\"").is_some() {
//                     memory.push(&mut Variable {
//                         name: name.to_owned(),
//                         types: "string".to_string(),
//                         value: Value::Chars(Chars {
//                             value: inputed.replace("'", "").replace("\"", ""),
//                         }),
//                     });
//                 } else {
//                     let value = compute(&inputed, memory, name_space);
//                     memory.push(&mut Variable {
//                         name: name.to_owned(),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value: value,
//                             expr: inputed,
//                         }),
//                     });
//                 }
//             } else if lines.find("print").is_some() {
//                 let new_lines = lines.replacen("print", "", 1);
//                 let mut text = String::new();
//                 let params = &new_lines;
//                 for i in params.split(",").collect::<Vec<&str>>() {
//                     if i.find("'").is_some() || i.find("\"").is_some() {
//                         //文字列か？
//                         text += &i.replace("'", "").replace("\"", "");
//                     } else {
//                         //文字列以外は式として扱われる
//                         text += &compute(&i.trim().to_string(), memory, name_space).to_string();
//                     }
//                 }
//                 println!("[出力]: {text}");
//             } else if lines.find("del").is_some() {
//                 let new_lines = lines.replacen("del", "", 1);
//                 let name = &new_lines;
//                 match reference_variable(name.clone(), memory) {
//                     Some(index) => {
//                         memory.remove(index);
//                         println!("変数{}を削除しました", name);
//                         continue;
//                     }
//                     None => {}
//                 }
//                 match reference_function(name.to_owned(), name_space) {
//                     Some(index) => {
//                         memory.remove(index);
//                         println!("関数{}を削除しました", name);
//                         continue;
//                     }
//                     None => {}
//                 }
//             } else if lines.find("#").is_some() {
//             } else if lines.find("rand").is_some() {
//                 let new_lines = lines.replacen("rand", "", 1);
//                 let params = new_lines.split(",").collect::<Vec<&str>>();
//                 if params.len() < 3 {
//                     let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
//                     let temp: i64 = rng.gen_range(1, 10);
//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value: temp as f64,
//                             expr: temp.to_string(),
//                         }),
//                     });
//                 } else {
//                     let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
//                     let temp: i64 = rng.gen_range(
//                         {
//                             println!("最小値を求めています");
//                             compute(&String::from(params[1]), memory, name_space).round() as i64
//                         },
//                         {
//                             println!("最大値を求めています");
//                             compute(&String::from(params[2]), memory, name_space).round() as i64
//                         },
//                     );
//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value: temp as f64,
//                             expr: temp.to_string(),
//                         }),
//                     });
//                 }
//                 println!("乱数を生成しました");
//             } else if lines.find("return").is_some() {
//                 let return_value = lines.replacen("return", "", 1);
//                 return Some(compute(&return_value, memory, name_space));
//             } else if lines.find("break").is_some() {
//                 return Some(f64::MAX);
//             } else if lines == "exit" {
//                 println!("終了します");
//                 exit(0);
//             } else if lines == "" {
//             } else {
//                 println!("コマンドが不正です: {}", lines)
//             }
//             remove_duplicates_variable(memory);
//             remove_duplicates_function(name_space);

//             // デバッグメニューを表示する
//             loop {
//                 let menu = input::input("デバッグメニュー>>> ");
//                 if menu.find("var").is_some() {
//                     let lim = &menu.replacen("var", "", 1);
//                     let params: Vec<&str> = lim.split("=").collect();
//                     let value = compute(&params[1..].join("=").to_string(), memory, name_space);

//                     memory.push(&mut Variable {
//                         name: params[0].trim().replace(" ", ""),
//                         types: "number".to_string(),
//                         value: Value::Number(Number {
//                             value,
//                             expr: params[1..].join("=").to_string(),
//                         }),
//                     });
//                 } else if menu.find("mem").is_some() {
//                     if memory.len() != 0 {
//                         println!("+-- メモリ内の変数 --");
//                         for i in memory.iter() {
//                             if i.types == "string" {
//                                 println!(
//                                     "| name: '{}' - type: {}型 -  - value: [{}]",
//                                     i.name,
//                                     i.types,
//                                     i.get_string().value
//                                 )
//                             } else {
//                                 println!(
//                                     "| name: '{}' - type: {}型 - expr: [{}] - value: {}",
//                                     i.name,
//                                     i.types,
//                                     i.get_number().expr,
//                                     i.get_number().value
//                                 )
//                             }
//                         }
//                     } else {
//                         println!("変数がありません");
//                     }
//                     if name_space.len() != 0 {
//                         println!("+-- メモリ内の関数 --");
//                         for i in name_space.iter() {
//                             println!("| name: '{}' - len: {}", i.name, i.code.len());
//                         }
//                     } else {
//                         println!("関数がありません");
//                     }
//                 } else if menu.find("exit").is_some() {
//                     input::input("デバッグを中断します");
//                     exit(0);
//                 } else {
//                     input::input("継続します");
//                     break;
//                 }
//                 remove_duplicates_variable(memory);
//                 remove_duplicates_function(name_space);
//             }
//         }
//     }
//     return None;
// }

// ///　変数の参照
// fn reference_variable(name: String, memory: &mut Vec<&mut Variable>) -> Option<usize> {
//     let name = name.trim().replace(" ", "");
//     match memory
//         .iter()
//         .position(|x| x.name == name.trim().replace(" ", ""))
//     {
//         Some(index) => Some(index),
//         None => {
//             println!("変数{name}が見つかりません");
//             None
//         }
//     }
// }

// ///　関数の参照
// fn reference_function(name: String, name_space: &mut Vec<Func>) -> Option<usize> {
//     let name = name.trim().replace(" ", "");
//     match name_space
//         .iter()
//         .position(|x| x.name == name.trim().replace(" ", ""))
//     {
//         Some(index) => Some(index),
//         None => {
//             println!("関数{name}が見つかりません");
//             None
//         }
//     }
// }

// /// 変数の参照(ログ出力なし)
// fn reference_variable_quiet(name: String, memory: &mut Vec<&mut Variable>) -> Option<usize> {
//     let name = name.trim().replace(" ", "");
//     match memory
//         .iter()
//         .position(|x| x.name == name.trim().replace(" ", ""))
//     {
//         Some(index) => Some(index),
//         None => None,
//     }
// }

// /// 関数の参照(ログ出力なし)
// fn reference_function_quiet(name: String, name_space: &mut Vec<Func>) -> Option<usize> {
//     let name = name.trim().replace(" ", "");
//     match name_space
//         .iter()
//         .position(|x| x.name == name.trim().replace(" ", ""))
//     {
//         Some(index) => Some(index),
//         None => None,
//     }
// }

// /// 逆ポーランド記法の式を計算する
// fn compute(expr: &String, memory: &mut Vec<&mut Variable>, name_space: &mut Vec<Func>) -> f64 {
//     let mut stack: Vec<f64> = Vec::new();
//     let tokens = expr.split_whitespace();
//     println!("+-- 計算処理 --");
//     for i in tokens {
//         let i = i.trim();
//         if i.len() == 0 {
//             continue;
//         }
//         println!("| Stack: {:?}  <=  '{}'", stack, i);
//         match i.parse::<f64>() {
//             Ok(num) => {
//                 stack.push(num);
//                 continue;
//             }
//             Err(_) => {
//                 let y = stack.pop().unwrap_or(0.0);
//                 let x = stack.pop().unwrap_or(0.0);
//                 match i {
//                     "+" => stack.push(x + y),
//                     "-" => stack.push(x - y),
//                     "*" => stack.push(x * y),
//                     "/" => stack.push(x / y),
//                     "%" => stack.push(x % y),
//                     "^" => stack.push(x.powf(y)),
//                     "=" => stack.push(if x == y { 1.0 } else { 0.0 }),
//                     "&" => stack.push(if x != 0.0 && y != 0.0 { 1.0 } else { 0.0 }),
//                     "|" => stack.push(if x != 0.0 || y != 0.0 { 1.0 } else { 0.0 }),
//                     ">" => stack.push(if x > y { 1.0 } else { 0.0 }),
//                     "<" => stack.push(if x < y { 1.0 } else { 0.0 }),
//                     "!" => {
//                         stack.push(x);
//                         stack.push(if y == 0.0 { 1.0 } else { 0.0 })
//                     }
//                     _ => {
//                         stack.push(x);
//                         stack.push(y);

//                         match reference_variable_quiet(i.to_string(), memory) {
//                             Some(i) => {
//                                 stack.push(memory[i].get_number().value);
//                             }
//                             None => {}
//                         }
//                         match reference_function_quiet(i.to_string(), name_space) {
//                             Some(index) => stack.push({
//                                 println!("関数{i}を呼び出します");
//                                 match execute(name_space[index].code.clone(), memory, name_space) {
//                                     Some(indes) => indes,
//                                     None => 0.0,
//                                 }
//                             }),
//                             None => {}
//                         }
//                     }
//                 }
//             }
//         };
//     }
//     let result = stack.pop().unwrap_or(0.0);
//     println!("結果 = {}", result);
//     return result;
// }

/// ファイルを読み込む
fn get_file_contents(name: String) -> Result<String, Error> {
    let mut f = File::open(name.trim())?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() {
    let message = "Simple プログラミング言語\nコンピュータの動作原理やロジックを学べます\n(c) 2023 梶塚太智. All rights reserved";
    let args = env::args().collect::<Vec<_>>();

    let mut executor = executor::Executor::new(&Vec::new(), &Vec::new());
    if args.len() >= 3 {
        //ファイルが環境変数にあるか?
        match get_file_contents(args[2].to_string()) {
            Ok(code) => {
                if args[1] == "run" || args[1] == "r" {
                    println!("{message}");
                    executor.execute_block(&code);
                } else if args[1] == "debug" || args[1] == "d" {
                    println!("{}をデバッグします", args[2]);
                    executor.debug(&code);
                } else if args[1] == "interactive" || args[1] == "i" {
                    println!("{message}");
                    executor.interactive();
                } else {
                    println!("実行モードを正しく指定してください")
                }
            }
            Err(e) => {
                eprintln!("エラー! :{}", e);
            }
        }
    } else if args.len() == 2 {
        if args[1] == "interactive" || args[1] == "i" {
            println!("{message}");
            executor.interactive();
        }
        match get_file_contents(args[1].to_string()) {
            Ok(code) => {
                executor.execute_block(&code);
            }
            Err(e) => {
                eprintln!("エラー! :{}", e);
            }
        }
    } else {
        //ファイルがない場合はインタラクティブで実行する
        println!("{message}");
        executor.interactive();
    }
}
