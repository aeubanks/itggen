use crate::generator::{Generator, GeneratorParameters};
use crate::style::Style;

fn to_lines(s: &str) -> Vec<String> {
    s.lines()
        .map(|s| {
            let mut s = s.to_owned();
            if let Some(p) = s.find("//") {
                s.truncate(p);
            }
            s.trim().to_owned()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

fn find_start_at(slice: &str, at: usize, pat: &str) -> Option<usize> {
    slice[at..].find(pat).map(|i| at + i)
}

fn columns(s: &str) -> Option<Vec<i8>> {
    let mut ret = Vec::new();
    for (i, c) in s.chars().enumerate() {
        if match c {
            '1' | '2' | '4' | 'L' => true,
            '0' | '3' | 'M' | 'F' => false,
            _ => return None,
        } {
            ret.push(i as i8);
        }
    }
    ret.truncate(2); // max 2 columns
    Some(ret)
}

fn generate_chart(
    notes_str: &str,
    from_style: Style,
    to_style: Style,
    params: GeneratorParameters,
) -> Result<String, String> {
    let mut ret = String::new();

    let lines = to_lines(notes_str);
    if lines.len() < 6 {
        return Err("invalid metadata".to_owned());
    }

    let (metadata, notes) = lines.split_at(6);
    if metadata[0] != "#NOTES:" {
        return Err(format!("expected '#NOTES:', got '{}'", metadata[0]));
    }
    ret.push_str("#NOTES:\n");
    if metadata[1] != format!("{}:", from_style.sm_string()) {
        println!(
            "skipping {} chart",
            &metadata[1][0..(metadata[1].len() - 1)]
        );
        return Ok("".to_owned());
    }
    ret.push_str("     ");
    ret.push_str(to_style.sm_string());
    ret.push_str(":\n     ");
    ret.push_str("AYEAG - ");
    ret.push_str(&metadata[2]);
    ret.push_str("\n     ");
    ret.push_str(&metadata[3]);
    ret.push_str("\n     ");
    ret.push_str(&metadata[4]);
    ret.push_str("\n     :\n");

    let mut gen = Generator::new(to_style, params);
    for l in notes {
        if let Some(cols) = columns(&l) {
            let mut out_cols = Vec::new();
            for col in cols {
                let idx = gen.gen_with_input_col(col);
                out_cols.push(idx);
            }
            let row = (0..(to_style.num_cols()))
                .map(|c| if out_cols.contains(&c) { '1' } else { '0' })
                .collect::<String>();
            ret.push_str(&row);
            ret.push('\n');
        } else if l == "," || l == ";" {
            ret.push_str(l);
            ret.push_str("\n");
        } else {
            return Err(format!("unknown notes line: {}", l));
        }
    }
    Ok(ret)
}

pub fn generate(
    contents: &str,
    from_style: Style,
    to_style: Style,
    params: GeneratorParameters,
) -> Result<String, String> {
    if contents.contains(to_style.sm_string()) {
        return Err(format!("already contains {} charts", to_style.sm_string()));
    }
    let mut ret = String::new();
    let mut search_from = 0;
    while let Some(notes_idx) = find_start_at(&contents, search_from, "#NOTES:") {
        let semicolon_idx = match find_start_at(&contents, notes_idx, ";") {
            Some(i) => i,
            None => {
                return Err("couldn't find semicolon after #NOTES".to_owned());
            }
        };
        let notes_str = &contents[notes_idx..=semicolon_idx];
        ret.push_str(&generate_chart(notes_str, from_style, to_style, params)?);
        search_from = semicolon_idx;
    }

    Ok(ret)
}

#[test]
fn test_generate() {
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            GeneratorParameters::default(),
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Challenge:\n     17:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Hard:\n     17:\n     useless:\n0000\n;\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            GeneratorParameters::default(),
        );
        assert_eq!(g, Ok("#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Hard:\n     17:\n     :\n00000000\n;\n#NOTES:\n     dance-double:\n     AYEAG - Zaia:\n     Challenge:\n     17:\n     :\n00000000\n;\n".to_owned()))
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge;\n     17:\n     useless:\n0000\n;".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            GeneratorParameters::default(),
        );
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n;\n#NOTES:\n     dance-double:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n00000000\n;\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            GeneratorParameters::default(),
        );
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            GeneratorParameters::default(),
        );
        assert!(g.is_err());
    }
    {
        let orig = "A\n#NOTES:\n     dance-single:\n     Zaia:\n     Challenge:\n     17:\n     useless:\n0000,0070;\n\n".to_owned();
        let g = generate(
            &orig,
            Style::ItgSingles,
            Style::ItgDoubles,
            GeneratorParameters::default(),
        );
        assert!(g.is_err());
    }
}
