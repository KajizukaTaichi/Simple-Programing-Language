use rand::Rng;
use std::env;
use std::fs::File;
use std::io::{Error, Read};
use std::process::exit;
mod input;

//変数のデータ
#[derive(Clone)]
struct Variable {
    name: String,
    expr: String, // 式
    value: f64,
}

// 関数のデータ
struct Func {
    name: String,
    code: String,
}

//メモリの重複を削除する
fn remove_duplicates(memory: &mut Vec<Variable>) -> &mut Vec<Variable> {
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

// REPLの対話実行
fn interactive(memory: &mut Vec<Variable>, name_space: &mut Vec<Func>) {
    loop {
        // 無限ループで「exit」コマンドまで永遠に実行
        let mut lines = input::input("プログラム>>> ");
        // 変数宣言
        if lines.find("var").is_some() {
            lines = lines.replacen("var", "", 1);
            let params: Vec<&str> = lines.split("=").collect();
            let value = compute(&params[1..].join("=").to_string(), memory, name_space);
            memory.push(Variable {
                name: params[0].trim().to_string(),
                value,
                expr: params[1..].join("=").to_string(),
            });
        //変数の式の再計算
        } else if lines.find("calc").is_some() {
            let name = lines.replacen("calc", "", 1);
            for index in 0..memory.len() {
                let value = compute(&memory[index].to_owned().expr, memory, name_space);
                if name.trim().to_string() == memory[index].name {
                    memory[index].value = value;
                    println!("再計算を実行しました");
                    break;
                }
            }
        // メモリのデータの表示
        } else if lines.find("mem").is_some() {
            if memory.len() != 0 {
                println!("+-- メモリ内の変数 --");
                for i in memory.iter() {
                    println!(
                        "| name: '{}' - expr: [{}] - value: {}",
                        i.name, i.expr, i.value
                    )
                }
            } else {
                println!("変数がありません");
            }
            if name_space.len() != 0 {
                println!("+-- メモリ内の関数 --");
                for i in name_space.iter() {
                    println!("|+-- name: '{}' - len: {}", i.name, i.code.len());
                    let mut number = 0; //行数
                    for j in i.code.split('\n') {
                        if j != "" {
                            number += 1;
                            println!("|| [{number}]: {j}");
                        }
                    }
                }
            } else {
                println!("関数がありません");
            }
        //forループ
        } else if lines.find("for").is_some() {
            // ループ回数は式で計算する
            println!("ループ回数を計算します");
            let index: i32 =
                compute(&lines.replacen("for", "", 1), memory, name_space).round() as i32;
            // 繰り返す関数
            let mut code = String::new();
            loop {
                let lines = input::input("forループ>>> ");
                if lines == "end for".to_string() {
                    break; //「end for」までループ
                }
                code += &lines;
                code += "\n";
            }

            for i in 0..index {
                println!("{}回目のループ", i + 1); //ループ実行
                execute(code.clone(), memory, name_space);
            }
        // 関数(コマンドの集合体)の定義
        } else if lines.find("func").is_some() {
            let name = lines.trim().replacen("func", "", 1);
            let mut code = String::new();
            loop {
                let lines = input::input("funcブロック>>>");
                if lines == "end func".to_string() {
                    break;
                }
                code += &String::from("\n");
                code += &lines;
            }
            name_space.push(Func {
                name: name.trim().to_string(),
                code,
            });
        // 関数の呼び出し
        } else if lines.find("call").is_some() {
            lines = lines.replacen("call", "", 1);
            let name = lines.trim();
            let code = match name_space.iter().position(|x| x.name == name.to_string()) {
                Some(index) => name_space[index].code.clone(),
                None => {
                    println!("関数{name}が見つかりません");
                    "".to_string();
                    continue;
                }
            };
            println!("関数{name}を呼び出します");
            execute(code.clone(), memory, name_space);
        // if文
        } else if lines.find("if").is_some() {
            lines = lines.replacen("if", "", 1);
            let mut code = String::new();
            let mut else_code = String::new();
            let expr = lines;
            'a: /* 脱出用 */ loop {
                let lines = input::input("ifブロック>>> ");
                if lines == "else" { //「else」からelseブロック
                    loop {
                        let lines = input::input("elseブロック>>> ");
                        if lines == "end if".to_string() {
                            break 'a; //「end if」でif文終わり
                        }
                        else_code += &String::from("\n");
                        else_code += &lines;
                    }
                }
                if lines == "end if".to_string() {
                    break 'a;
                }
                code += &String::from("\n");
                code += &lines;
            }
            println!("ifの条件式を評価します");
            if compute(&expr, memory, name_space) != 0.0 {
                println!("条件が一致したので、実行します");
                execute(code, memory, name_space);
            } else {
                if else_code != "" {
                    // elseブロックがあるか?
                    println!("条件が一致しなかったので、elseのコードを実行します");
                    execute(else_code, memory, name_space);
                } else {
                    println!("条件が一致しなかったので、実行しません");
                }
            }
        // whileループ
        } else if lines.find("while").is_some() {
            let mut code = String::new(); //条件式
            let expr = lines.replacen("while", "", 1);
            loop {
                let lines = input::input("whileループ>>> ");
                if lines == "end while".to_string() {
                    break;
                }
                code += &lines;
                code += "\n";
            }
            loop {
                println!("whileの条件式を評価します");
                if compute(&expr.to_string(), memory, name_space) == 0.0 {
                    println!("条件が一致しなかったので、ループを脱出します");
                    break;
                } else {
                    println!("条件が一致したので、ループを継続します");
                    execute(code.clone(), memory, name_space);
                }
            }
        // input文(標準入力)
        } else if lines.find("input").is_some() {
            let name = lines.replacen("input", "", 1);

            let inputed = input::input("[入力]> ");
            let value = compute(&inputed, memory, name_space);
            memory.push(Variable {
                name: name.trim().to_string(),
                value: value,
                expr: inputed, //入力値は式として扱われる
            });
        // コメント
        } else if lines.find("#").is_some() {
            // print文(標準出力)
        } else if lines.find("print").is_some() {
            lines = lines.replacen("print", "", 1);
            let mut text: String = String::new();
            let params = lines;
            for i in params.split(",").collect::<Vec<&str>>() {
                if i.find("'").is_some() {
                    //文字列か？
                    text += &i.replace("'", "");
                } else {
                    //文字列以外は式として扱われる
                    text += &compute(&i.trim().to_string(), memory, name_space).to_string();
                }
            }
            println!("[出力]: {text}");
        // delコマンド(変数の削除)
        } else if lines.find("del").is_some() {
            let name = lines.replacen("del", "", 1);
            // 変数の参照
            for index in 0..memory.len() {
                if name.to_string().trim() == memory[index].name {
                    memory.remove(index);
                    println!("変数を削除しました");
                    break;
                }
            }
            for index in 0..name_space.len() {
                if name.to_string().trim() == name_space[index].name {
                    name_space.remove(index);
                    println!("関数を削除しました");
                    break;
                }
            }
        } else if lines.find("rand").is_some() {
            lines = lines.replacen("rand", "", 1);
            let params = lines.split(",").collect::<Vec<&str>>();
            if params.len() < 1 {
                println!("エラー！変数を指定してください");
            } else if params.len() < 3 {
                let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                let temp: i64 = rng.gen_range(1, 10);
                memory.push(Variable {
                    name: params[0].trim().to_string(),
                    value: temp as f64,
                    expr: temp.to_string(),
                });
            } else if params.len() < 1 {
                println!("エラー！引数が多すぎます");
            } else {
                let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                let temp: i64 = rng.gen_range(
                    compute(&String::from(params[1]), memory, name_space).round() as i64,
                    compute(&String::from(params[2]), memory, name_space).round() as i64,
                );
                memory.push(Variable {
                    name: params[0].trim().to_string(),
                    value: temp as f64,
                    expr: temp.to_string(),
                });
            }
        } else if lines == "exit" {
            println!("終了します");
            exit(0);
        } else if lines == "" {
        } else {
            println!("コマンドが不正です: {}", lines)
        }
        remove_duplicates(memory);
    }
}

//　関数を一括実行
fn execute(code: String, memory: &mut Vec<Variable>, name_space: &mut Vec<Func>) -> f64 {
    let mut stmt = String::new(); // ブロックのステートメント
    let mut else_stmt = String::new(); // elseステートメント
    let mut count = 0; // ループカウンタ
    let mut name = String::new(); // 関数の名前
    let mut expr = String::new(); // 条件式
    let mut mode = String::from("normal"); // 制御ブロックの状態
    let mut old_mode = String::new(); // 元のモード
    let mut number = 0; //実行している行

    // 改行区切りで実行する
    for mut lines in code.split("\n") {
        lines = lines.trim_start().trim_end();
        if lines == "" {
            continue;
        } // 空白の行を飛ばす
        number += 1; // 進捗メッセージ
        println!("-- {number}行: [{lines}]を実行");
        // forモードの場合
        if mode == "for".to_string() {
            if lines == "end for" {
                // 「end for」でループ終わり
                for i in 0..count {
                    println!("{}回目のループ", i + 1); //ループ実行
                    execute(code.clone(), memory, name_space);
                }
            } else {
                stmt += lines;
                stmt += "\n";
            }
        } else if mode == "if".to_string() {
            if lines == "else" {
                old_mode = mode;
                mode = "else".to_string()
            } else if lines == "end if" {
                println!("ifの条件式を評価します");
                if compute(&expr, memory, name_space) != 0.0 {
                    println!("条件が一致したので、実行します");
                    execute(stmt.clone(), memory, name_space);
                    stmt = String::new();
                } else {
                    println!("条件が一致しなかったので、実行しません");
                    stmt = String::new();
                }
                mode = old_mode.clone();
            } else {
                stmt += lines;
                stmt += "\n";
            }
        } else if mode == "func".to_string() {
            if lines == "end func" {
                name_space.push(Func {
                    name: name.clone(),
                    code: stmt.clone(),
                });
                stmt = String::new();
                mode = old_mode.clone();
            } else {
                stmt += lines;
                stmt += &String::from("\n");
            }
        } else if mode == "else".to_string() {
            if lines == "end if" {
                println!("ifの条件式を評価します");
                if compute(&expr, memory, name_space) == 0.0 {
                    println!("条件が一致しなかったので、elseのコードを実行します");
                    execute(else_stmt.clone(), memory, name_space);
                    else_stmt = String::new();
                    stmt = String::new();
                } else {
                    println!("条件が一致したので、実行します");
                    execute(stmt.clone(), memory, name_space);
                    else_stmt = String::new();
                    stmt = String::new();
                }
                mode = old_mode.clone();
            } else {
                else_stmt += lines;
                else_stmt += &String::from("\n");
            }
        } else if mode == "while".to_string() {
            if lines == "end while" {
                loop {
                    println!("whileの条件式を評価します");
                    if compute(&expr, memory, name_space) == 0.0 {
                        println!("条件が一致しなかったので、ループを脱出します");
                        stmt = String::new();
                        break;
                    }
                    execute(stmt.clone(), memory, name_space);
                }
                mode = old_mode.clone();
            } else {
                stmt += lines;
                stmt += "\n";
            }
        } else {
            if lines.find("var").is_some() {
                let new_lines = lines.replacen("var", "", 1); // Create a new String
                lines = &new_lines;
                let params: Vec<&str> = lines.split("=").collect();
                let value = compute(&params[1..].join("=").to_string(), memory, name_space);
                memory.push(Variable {
                    name: params[0].trim().to_string(),
                    value: value,
                    expr: params[1..].join("=").to_string(),
                });
            } else if lines.find("calc").is_some() {
                let new_lines = lines.replacen("calc", "", 1); // Create a new String
                let name = &new_lines;
                for index in 0..memory.len() {
                    if name.to_string() == memory[index].name {
                        memory[index].value =
                            compute(&memory[index].to_owned().expr, memory, name_space);
                        println!("再計算を実行しました");
                        break;
                    }
                }
            } else if lines.find("mem").is_some() {
                if memory.len() != 0 {
                    println!("+-- メモリ内の変数 --");
                    for i in memory.iter() {
                        println!(
                            "| name: '{}' - expr: [{}] - value: {}",
                            i.name, i.expr, i.value
                        )
                    }
                } else {
                    println!("変数がありません");
                }
                if name_space.len() != 0 {
                    println!("+-- メモリ内の関数 --");
                    for i in name_space.iter() {
                        println!("|+-- name: '{}' - len: {}", i.name, i.code.len());
                        let mut number = 0; //行数
                        for j in i.code.split('\n') {
                            if j != "" {
                                number += 1;
                                println!("|| [{number}]: {j}");
                            }
                        }
                    }
                } else {
                    println!("関数がありません");
                }
            } else if lines.find("func").is_some() {
                let new_lines = lines.replacen("func", "", 1); // Create a new String
                name = new_lines;
                mode = "func".to_string();
            } else if lines.find("call").is_some() {
                let new_lines = lines.replacen("call", "", 1); // Create a new String
                let name = &new_lines;
                let codes = match name_space.iter().position(|x| x.name == name.to_string()) {
                    Some(index) => name_space[index].code.clone(),
                    None => {
                        println!("関数{name}が見つかりません");
                        "".to_string();
                        continue;
                    }
                };
                println!("関数{name}を呼び出します");
                execute(codes.clone(), memory, name_space);
            } else if lines.find("for").is_some() {
                let new_lines = lines.replacen("for", "", 1); // Create a new String
                count = compute(&new_lines, memory, name_space) as i32;
                old_mode = mode;
                mode = "for".to_string();
            } else if lines.find("if").is_some() {
                let new_lines = lines.replacen("if", "", 1); // Create a new String
                expr = new_lines;
                old_mode = mode;
                mode = "if".to_string()
            } else if lines.find("while").is_some() {
                let new_lines = lines.replacen("while", "", 1); // Create a new String
                expr = new_lines;
                old_mode = mode;
                mode = "while".to_string();
            } else if lines.find("input").is_some() {
                let new_lines = lines.replacen("input", "", 1); // Create a new String
                let name = &new_lines;

                let inputed = input::input("[入力]> ");
                let value = compute(&inputed, memory, name_space);
                memory.push(Variable {
                    name: name.trim().to_string(),
                    value: value,
                    expr: inputed,
                });
            } else if lines.find("print").is_some() {
                let new_lines = lines.replacen("print", "", 1); // Create a new String
                let mut text = String::new();
                let params = &new_lines;
                for i in params.split(",").collect::<Vec<&str>>() {
                    if i.find("'").is_some() {
                        //文字列か？
                        text += &i.replace("'", "");
                    } else {
                        //文字列以外は式として扱われる
                        text += &compute(&i.trim().to_string(), memory, name_space).to_string();
                    }
                }
                println!("[出力]: {text}");
            } else if lines.find("rand").is_some() {
                let new_lines = lines.replacen("rand", "", 1);
                let params = new_lines.split(",").collect::<Vec<&str>>();
                if params.len() < 1 {
                    println!("エラー！変数を指定してください");
                } else if params.len() < 3 {
                    let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                    let temp: i64 = rng.gen_range(1, 10);
                    memory.push(Variable {
                        name: params[0].trim().to_string(),
                        value: temp as f64,
                        expr: temp.to_string(),
                    });
                } else if params.len() < 1 {
                    println!("エラー！引数が多すぎます");
                } else {
                    let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                    let temp: i64 = rng.gen_range(
                        compute(&String::from(params[1]), memory, name_space).round() as i64,
                        compute(&String::from(params[2]), memory, name_space).round() as i64,
                    );
                    memory.push(Variable {
                        name: params[0].trim().to_string(),
                        value: temp as f64,
                        expr: temp.to_string(),
                    });
                }
            } else if lines.find("del").is_some() {
                let new_lines = lines.replacen("del", "", 1); // Create a new String
                let name = &new_lines;
                for index in 0..memory.len() {
                    if name.to_string() == memory[index].name {
                        memory.remove(index);
                        println!("変数を削除しました");
                        break;
                    }
                }
            } else if lines.find("return").is_some() {
                let return_value = lines.replacen("return", "", 1); // Create a new String
                return compute(&return_value, memory, name_space);
            } else if lines.find("#").is_some() {
            } else if lines == "exit" {
                println!("終了します");
                exit(0);
            } else if lines == "" {
            } else {
                println!("コマンドが不正です: {}", lines)
            }
            remove_duplicates(memory);
        }
    }
    return 0.0;
}

// ファイルに保存されたスクリプトを実行
fn script(code: String, memory: &mut Vec<Variable>, name_space: &mut Vec<Func>) -> f64 {
    let mut stmt = String::new(); // ブロックのステートメント
    let mut else_stmt = String::new(); // elseステートメント
    let mut count = 0; // ループカウンタ
    let mut name = String::new(); // 関数の名前
    let mut expr = String::new(); // 条件式
    let mut mode = String::from("normal"); // 制御ブロックの状態
    let mut old_mode = String::new(); // 元のモード
    let mut nest_if = 0; // ifネストの階層を表す
    let mut nest_for = 0; // forネストの階層を表す
    let mut nest_while = 0; // whileネストの階層を表す
    let mut nest_code = 0; // codeネストの階層を表す

    for mut lines in code.split("\n") {
        lines = lines.trim_start().trim_end();
        if mode == "for".to_string() {
            if lines.find("end for").is_some() {
                if nest_for > 0 {
                    nest_for -= 1;
                    stmt += lines;
                    stmt += "\n";
                } else {
                    for _ in 0..count {
                        script(code.clone(), memory, name_space);
                    }
                    stmt = String::new();
                    mode = old_mode.clone();
                }
            } else if lines.find("for").is_some() {
                nest_for += 1;
                stmt += lines;
                stmt += "\n";
            } else {
                stmt += lines;
                stmt += "\n";
            }
        } else if mode == "if".to_string() {
            if lines.find("else").is_some() {
                old_mode = mode;
                mode = "else".to_string()
            } else if lines.find("end if").is_some() {
                if nest_if > 0 {
                    nest_if -= 1;
                    stmt += lines;
                    stmt += "\n";
                } else {
                    if calculation(&expr, memory, name_space) != 0.0 {
                        script(stmt.clone(), memory, name_space);
                        stmt = String::new();
                    } else {
                        stmt = String::new();
                    }
                    mode = old_mode.clone();
                }
            } else if lines.find("if").is_some() {
                nest_if += 1;
                stmt += lines;
                stmt += "\n";
            } else {
                stmt += lines;
                stmt += "\n";
            }
        } else if mode == "func".to_string() {
            if lines.find("end func").is_some() {
                if nest_code > 0 {
                    nest_code -= 1;
                } else {
                    name_space.push(Func {
                        name: name.clone(),
                        code: stmt.clone(),
                    });
                    stmt = String::new();
                    mode = old_mode.clone();
                }
            } else if lines.find("func").is_some() {
                nest_code += 1;
            } else {
                stmt += lines;
                stmt += &String::from("\n");
            }
        } else if mode == "else".to_string() {
            if lines.find("end if").is_some() {
                if nest_if > 0 {
                    nest_if -= 1;
                    stmt += lines;
                    stmt += "\n";
                } else {
                    if calculation(&expr, memory, name_space) == 0.0 {
                        script(else_stmt.clone(), memory, name_space);
                        else_stmt = String::new();
                        stmt = String::new();
                    } else {
                        script(stmt.clone(), memory, name_space);
                        else_stmt = String::new();
                        stmt = String::new();
                    }
                    mode = old_mode.clone();
                }
            } else if lines.find("if").is_some() {
                nest_if += 1;
                stmt += lines;
                stmt += "\n";
                mode = "if".to_string();
            } else {
                else_stmt += lines;
                else_stmt += &String::from("\n");
            }
        } else if mode == "while".to_string() {
            if lines.find("end while").is_some() {
                if nest_while > 0 {
                    nest_while -= 1;
                } else {
                    loop {
                        if calculation(&expr, memory, name_space) == 0.0 {
                            stmt = String::new();
                            break;
                        } else {
                            script(stmt.clone(), memory, name_space);
                        }
                    }
                    mode = old_mode.clone();
                }
            } else if lines.find("while").is_some() {
                nest_while += 1;
            } else {
                stmt += lines;
                stmt += "\n";
            }
        } else {
            if lines.find("var").is_some() {
                let new_lines = lines.replacen("var", "", 1); // Create a new String
                lines = &new_lines;
                let params: Vec<&str> = lines.split("=").collect();
                let value = calculation(&params[1..].join("").to_string(), memory, name_space);
                memory.push(Variable {
                    name: params[0].trim().to_string(),
                    value: value,
                    expr: params[1..].join("=").to_string(),
                });
            } else if lines.find("calc").is_some() {
                let new_lines = lines.replacen("calc", "", 1); // Create a new String
                let name = &new_lines;
                for index in 0..memory.len() {
                    if name.to_string() == memory[index].name {
                        memory[index].value =
                            calculation(&memory[index].to_owned().expr, memory, name_space);
                        break;
                    }
                }
            } else if lines.find("func").is_some() {
                let new_lines = lines.trim().replacen("func", "", 1); // Create a new String
                name = new_lines;
                mode = "func".to_string();
            } else if lines.find("call").is_some() {
                let new_lines = lines.replacen("call", "", 1); // Create a new String
                let name = &new_lines;
                let code = match name_space.iter().position(|x| x.name == name.to_string()) {
                    Some(index) => name_space[index].code.clone(),
                    None => {
                        println!("関数{name}が見つかりません");
                        "".to_string();
                        continue;
                    }
                };
                script(code.clone(), memory, name_space);
            } else if lines.find("for").is_some() {
                let new_lines = lines.replacen("for", "", 1); // Create a new String
                count = calculation(&new_lines, memory, name_space) as i32;
                old_mode = mode;
                mode = "for".to_string();
            } else if lines.find("if").is_some() {
                let new_lines = lines.replacen("if", "", 1); // Create a new String
                expr = new_lines;
                old_mode = mode;
                mode = "if".to_string()
            } else if lines.find("while").is_some() {
                let new_lines = lines.replacen("while", "", 1); // Create a new String
                expr = new_lines;
                old_mode = mode;
                mode = "while".to_string();
            } else if lines.find("input").is_some() {
                let new_lines = lines.replacen("input", "", 1); // Create a new String
                let name = &new_lines;

                let inputed = input::input("> ");
                let value = calculation(&inputed, memory, name_space);
                memory.push(Variable {
                    name: name.trim().to_string(),
                    value: value,
                    expr: inputed,
                });
            } else if lines.find("print").is_some() {
                let new_lines = lines.replacen("print", "", 1); // Create a new String
                let mut text = String::new();
                let params = &new_lines;
                for i in params.split(",").collect::<Vec<&str>>() {
                    if i.find("'").is_some() {
                        //文字列か？
                        text += &i.replace("'", "");
                    } else {
                        //文字列以外は式として扱われる
                        text += &calculation(&i.trim().to_string(), memory, name_space).to_string();
                    }
                }
                println!("{text}");
            } else if lines.find("del").is_some() {
                let new_lines = lines.replacen("del", "", 1); // Create a new String
                let name = &new_lines;
                for index in 0..memory.len() {
                    if name.to_string() == memory[index].name {
                        memory.remove(index);
                        break;
                    }
                }
            } else if lines.find("#").is_some() {
            } else if lines.find("rand").is_some() {
                let new_lines = lines.replacen("rand", "", 1);
                let params = new_lines.split(",").collect::<Vec<&str>>();
                if params.len() < 1 {
                    println!("エラー！変数を指定してください");
                } else if params.len() < 3 {
                    let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                    let temp: i64 = rng.gen_range(1, 10);
                    memory.push(Variable {
                        name: params[0].trim().to_string(),
                        value: temp as f64,
                        expr: temp.to_string(),
                    });
                } else if params.len() < 1 {
                    println!("エラー！引数が多すぎます");
                } else {
                    let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                    let temp: i64 = rng.gen_range(
                        calculation(&String::from(params[1]), memory, name_space).round() as i64,
                        calculation(&String::from(params[2]), memory, name_space).round() as i64,
                    );
                    memory.push(Variable {
                        name: params[0].trim().to_string(),
                        value: temp as f64,
                        expr: temp.to_string(),
                    });
                }
            } else if lines.find("return").is_some() {
                let return_value = lines.replacen("return", "", 1); // Create a new String
                return calculation(&return_value, memory, name_space);
            } else if lines == "exit" {
                exit(0);
            } else if lines == "" {
            } else {
                println!("コマンドが不正です: {}", lines)
            }
            remove_duplicates(memory);
        }
    }
    return 0.0;

    fn calculation(expr: &String, memory: &mut Vec<Variable>, name_space: &mut Vec<Func>) -> f64 {
        let mut stack: Vec<f64> = Vec::new();
        let tokens = expr.split(' ');
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
                Err(_) => match memory.iter().position(|x| x.name == i.to_string()) {
                    Some(index) => {
                        stack.push(memory[index].value); //　メモリを参照
                    }
                    None => {
                        match name_space.iter().position(|x| x.name == i.to_string()) {
                            Some(index) => {
                                stack.push(script(
                                    name_space[index].code.clone(), //　関数呼び出し
                                    memory,
                                    name_space,
                                ));
                            }
                            None => {
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
                                    }
                                }
                            }
                        };
                    }
                },
            };
        }
        let result = stack.pop().unwrap_or(0.0);
        return result;
    }
}
fn debug(code: String, memory: &mut Vec<Variable>, name_space: &mut Vec<Func>) -> f64 {
    let mut stmt = String::new(); // ブロックのステートメント
    let mut else_stmt = String::new(); // elseステートメント
    let mut count = 0; // ループカウンタ
    let mut name = String::new(); // 関数の名前
    let mut expr = String::new(); // 条件式
    let mut mode = String::from("normal"); // 制御ブロックの状態
    let mut old_mode = String::new(); // 元のモード
    let mut number = 0; //実行している行
    let mut nest_if = 0; // ifネストの階層を表す
    let mut nest_for = 0; // forネストの階層を表す
    let mut nest_while = 0; // whileネストの階層を表す
    let mut nest_code = 0; // codeネストの階層を表す

    // 改行区切りで実行する
    for mut lines in code.split("\n") {
        lines = lines.trim_start().trim_end();
        if lines == "" {
            continue;
        } // 空白の行を飛ばす
        number += 1; // 進捗メッセージ
                     // forモードの場合
        if mode == "for".to_string() {
            if lines == "end for" {
                if nest_for > 0 {
                    nest_for -= 1;
                    stmt += lines;
                    stmt += "\n";
                } else {
                    // 「end for」でループ終わり
                    for _ in 0..count {
                        debug(code.clone(), memory, name_space);
                    }
                }
                stmt = String::new();
                mode = old_mode.clone();
            } else if lines.find("for").is_some() {
                nest_for += 1;
                stmt += lines;
                stmt += "\n";
            } else {
                stmt += lines;
                stmt += "\n";
            }
        } else if mode == "if".to_string() {
            if lines == "else" {
                old_mode = mode;
                mode = "else".to_string()
            } else if lines == "end if" {
                if nest_if > 0 {
                    nest_if -= 1;
                    stmt += lines;
                    stmt += "\n";
                } else {
                    println!("ifの条件式を評価します");
                    if compute(&expr, memory, name_space) != 0.0 {
                        println!("条件が一致したので、実行します");
                        debug(stmt.clone(), memory, name_space);
                        stmt = String::new();
                    } else {
                        println!("条件が一致しなかったので、実行しません");
                        stmt = String::new();
                    }
                }
                mode = old_mode.clone();
            } else if lines.find("if").is_some() {
                nest_if += 1;
                stmt += lines;
                stmt += "\n";
            } else {
                stmt += lines;
                stmt += "\n";
            }
        } else if mode == "func".to_string() {
            if lines == "end func" {
                if nest_code > 0 {
                    nest_code -= 1;
                    stmt += lines;
                    stmt += "\n";
                } else {
                    name_space.push(Func {
                        name: name.clone(),
                        code: stmt.clone(),
                    });
                    stmt = String::new();
                    mode = old_mode.clone();
                }
            } else if lines.find("func").is_some() {
                nest_code += 1;
                stmt += lines;
                stmt += "\n";
            } else {
                stmt += lines;
                stmt += &String::from("\n");
            }
        } else if mode == "else".to_string() {
            if lines == "end if" {
                if nest_if > 0 {
                    nest_if -= 1;
                    stmt += lines;
                    stmt += "\n";
                } else {
                    println!("ifの条件式を評価します");
                    if compute(&expr, memory, name_space) == 0.0 {
                        println!("条件が一致しなかったので、elseのコードを実行します");
                        debug(else_stmt.clone(), memory, name_space);
                        else_stmt = String::new();
                        stmt = String::new();
                    } else {
                        println!("条件が一致したので、実行します");
                        debug(stmt.clone(), memory, name_space);
                        else_stmt = String::new();
                        stmt = String::new();
                    }
                }
                mode = old_mode.clone();
            } else if lines.find("if").is_some() {
                nest_if += 1;
                stmt += lines;
                stmt += "\n";
            } else {
                else_stmt += lines;
                else_stmt += &String::from("\n");
            }
        } else if mode == "while".to_string() {
            if lines == "end while" {
                if nest_while > 0 {
                    nest_while -= 1;
                    stmt += lines;
                    stmt += "\n";
                } else {
                    loop {
                        println!("whileの条件式を評価します");
                        if compute(&expr, memory, name_space) == 0.0 {
                            println!("条件が一致しなかったので、ループを脱出します");
                            stmt = String::new();
                            break;
                        }
                        debug(stmt.clone(), memory, name_space);
                    }
                }
                mode = old_mode.clone();
            } else if lines.find("while").is_some() {
                nest_while += 1;
                stmt += lines;
                stmt += "\n";
            } else {
                stmt += lines;
                stmt += "\n";
            }
        } else {
            println!("+- {number}行: [{lines}]を実行");
            if lines.find("var").is_some() {
                let new_lines = lines.replacen("var", "", 1); // Create a new String
                lines = &new_lines;
                let params: Vec<&str> = lines.split("=").collect();
                let value = compute(&params[1..].join("=").to_string(), memory, name_space);
                memory.push(Variable {
                    name: params[0].trim().to_string(),
                    value: value,
                    expr: params[1..].join("=").to_string(),
                });
            } else if lines.find("calc").is_some() {
                let new_lines = lines.replacen("calc", "", 1); // Create a new String
                let name = &new_lines;
                for index in 0..memory.len() {
                    if name.to_string() == memory[index].name {
                        memory[index].value =
                            compute(&memory[index].to_owned().expr, memory, name_space);
                        println!("再計算を実行しました");
                        break;
                    }
                }
            } else if lines.find("mem").is_some() {
                println!("+-- メモリ内の変数 --");
                for i in memory.iter() {
                    println!(
                        "| name: '{}' - expr: [{}] - value: {}",
                        i.name, i.expr, i.value
                    );
                }
                println!("+-- メモリ内の関数 --");
                for i in name_space.iter() {
                    println!("| name: '{}'", i.name);
                    let mut number = 0; //行数
                    for j in i.code.split('\n') {
                        number += 1;
                        println!("|| [{number}]: {j}");
                    }
                }
            } else if lines.find("func").is_some() {
                name = lines.trim().replacen("func", "", 1); // Create a new String
                mode = "func".to_string();
                println!("関数{name}を定義します");
            } else if lines.find("call").is_some() {
                name = lines.trim().replacen("call", "", 1); // Create a new String
                let codes = match name_space.iter().position(|x| x.name == name.to_string()) {
                    Some(index) => name_space[index].code.clone(),
                    None => {
                        println!("関数{name}が見つかりません");
                        "".to_string();
                        continue;
                    }
                };
                println!("関数{name}を呼び出します");
                debug(codes.clone(), memory, name_space);
            } else if lines.find("for").is_some() {
                let new_lines = lines.replacen("for", "", 1); // Create a new String
                count = compute(&new_lines, memory, name_space) as i32;
                old_mode = mode;
                mode = "for".to_string();
            } else if lines.find("if").is_some() {
                let new_lines = lines.replacen("if", "", 1); // Create a new String
                expr = new_lines;
                old_mode = mode;
                mode = "if".to_string()
            } else if lines.find("while").is_some() {
                let new_lines = lines.replacen("while", "", 1); // Create a new String
                expr = new_lines;
                old_mode = mode;
                mode = "while".to_string();
            } else if lines.find("input").is_some() {
                let new_lines = lines.replacen("input", "", 1); // Create a new String
                let name = &new_lines;

                let inputed = input::input("[入力]> ");
                let value = compute(&inputed, memory, name_space);
                memory.push(Variable {
                    name: name.trim().to_string(),
                    value: value,
                    expr: inputed,
                });
            } else if lines.find("print").is_some() {
                let new_lines = lines.replacen("print", "", 1); // Create a new String
                let mut text = String::new();
                let params = &new_lines;
                for i in params.split(",").collect::<Vec<&str>>() {
                    if i.find("'").is_some() {
                        //文字列か？
                        text += &i.replace("'", "");
                    } else {
                        //文字列以外は式として扱われる
                        text += &compute(&i.trim().to_string(), memory, name_space).to_string();
                    }
                }
                println!("[出力]: {text}");
            } else if lines.find("del").is_some() {
                let new_lines = lines.replacen("del", "", 1); // Create a new String
                let name = &new_lines;
                for index in 0..memory.len() {
                    if name.to_string() == memory[index].name {
                        memory.remove(index);
                        println!("変数を削除しました");
                        break;
                    }
                }
            } else if lines.find("#").is_some() {
            } else if lines.find("rand").is_some() {
                let new_lines = lines.replacen("rand", "", 1);
                let params = new_lines.split(",").collect::<Vec<&str>>();
                if params.len() < 1 {
                    println!("エラー！変数を指定してください");
                } else if params.len() < 3 {
                    let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                    let temp: i64 = rng.gen_range(1, 10);
                    memory.push(Variable {
                        name: params[0].trim().to_string(),
                        value: temp as f64,
                        expr: temp.to_string(),
                    });
                } else if params.len() < 1 {
                    println!("エラー！引数が多すぎます");
                } else {
                    let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                    let temp: i64 = rng.gen_range(
                        compute(&String::from(params[1]), memory, name_space).round() as i64,
                        compute(&String::from(params[2]), memory, name_space).round() as i64,
                    );
                    memory.push(Variable {
                        name: params[0].trim().to_string(),
                        value: temp as f64,
                        expr: temp.to_string(),
                    });
                }
            } else if lines.find("return").is_some() {
                let return_value = lines.replacen("return", "", 1); // Create a new String
                return compute(&return_value, memory, name_space);
            } else if lines == "exit" {
                println!("終了します");
                exit(0);
            } else if lines == "" {
            } else {
                println!("コマンドが不正です: {}", lines)
            }
            remove_duplicates(memory);

            let menu = input::input("デバッグメニュー>>> ");
            if menu.find("mem").is_some() {
                if memory.len() != 0 {
                    println!("+-- メモリ内の変数 --");
                    for i in memory.iter() {
                        println!(
                            "| name: '{}' - expr: [{}] - value: {}",
                            i.name, i.expr, i.value
                        )
                    }
                } else {
                    println!("変数がありません");
                }
                if name_space.len() != 0 {
                    println!("+-- メモリ内の関数 --");
                    for i in name_space.iter() {
                        println!("| name: '{}' - len: {}", i.name, i.code.len());
                    }
                } else {
                    println!("関数がありません");
                }
                input::input("継続します");
            } else if menu.find("exit").is_some() {
                println!("デバッグを中断します");
                exit(0);
            } else {
                println!("継続します");
            }
        }
    }
    return 0.0;
}

fn compute(expr: &String, memory: &mut Vec<Variable>, name_space: &mut Vec<Func>) -> f64 {
    let mut stack: Vec<f64> = Vec::new();
    let tokens = expr.split(' ');
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
            Err(_) => match memory.iter().position(|x| x.name == i.to_string()) {
                Some(index) => {
                    stack.push(memory[index].value);
                }
                None => {
                    match name_space.iter().position(|x| x.name == i.to_string()) {
                        Some(index) => {
                            println!("関数{i}を呼び出します");
                            stack.push(execute(name_space[index].code.clone(), memory, name_space));
                        }
                        None => {
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
                                    println!("[ERROR] this operator is invalid \"{}\"", i);
                                    stack.push(x);
                                    stack.push(y);
                                }
                            }
                        }
                    };
                }
            },
        };
    }
    let result = stack.pop().unwrap_or(0.0);
    println!("結果 = {}", result);
    return result;
}

// ファイルを読み込む
fn get_file_contents(name: String) -> Result<String, Error> {
    let mut f = File::open(name.trim())?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() >= 3 {
        //ファイルが環境変数にあるか?
        match get_file_contents(args[2].to_string()) {
            Ok(func) => {
                if args[1] == "run" {
                    script(func, &mut Vec::new(), &mut Vec::new());
                } else if args[1] == "debug" {
                    println!("{}をデバッグします", args[2]);
                    debug(func, &mut Vec::new(), &mut Vec::new());
                } else if args[1] == "interactive" {
                    println!("Simple プログラミング言語");
                    println!("コンピュータの動作原理やロジックを学べます");
                    println!("(c) 2023 梶塚太智. All rights reserved");
                    interactive(&mut Vec::new(), &mut Vec::new());
                } else {
                    println!("実行モードを正しく指定してください")
                }
            }
            Err(e) => {
                eprintln!("エラー! :{}", e);
            }
        }
    } else if args.len() == 2 {
        if args[1] == "interactive" {
            println!("Simple プログラミング言語");
            println!("コンピュータの動作原理やロジックを学べます");
            println!("(c) 2023 梶塚太智. All rights reserved");
            interactive(&mut Vec::new(), &mut Vec::new());
        }
        match get_file_contents(args[1].to_string()) {
            Ok(func) => {
                script(func, &mut Vec::new(), &mut Vec::new());
            }
            Err(e) => {
                eprintln!("エラー! :{}", e);
            }
        }
    } else {
        //ファイルがない場合はインタラクティブで実行する
        println!("Simple プログラミング言語");
        println!("コンピュータの動作原理やロジックを学べます");
        println!("(c) 2023 梶塚太智. All rights reserved");
        interactive(&mut Vec::new(), &mut Vec::new());
    }
}
