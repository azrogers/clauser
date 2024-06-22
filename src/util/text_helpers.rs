pub struct CharHelper<'a>(pub &'a Vec<char>);

const NEW_LINE: char = '\n';
const TAB: char = '\t';

impl<'a> CharHelper<'a> {
    /// Find the index of the first character of this line.
    /// If `position` points to a line break, it will be counted as the end of the previous line,
    /// rather than the start of a new one.
    pub fn find_line_start(&self, position: usize) -> usize {
        let mut current_pos = usize::min(position, self.0.len() - 1);

        if self.0[current_pos] == NEW_LINE {
            if current_pos == 0 {
                return current_pos;
            }

            current_pos = current_pos - 1;
        }

        loop {
            if self.0[current_pos] == NEW_LINE {
                return current_pos + 1;
            }

            if current_pos == 0 {
                break;
            }

            current_pos = current_pos - 1;
        }

        0
    }

    /// Find the index of the last character of the line, usually the line break.
    pub fn find_line_end(&self, position: usize) -> usize {
        let mut current_pos = position;
        while current_pos < self.0.len() {
            if self.0[current_pos] == NEW_LINE {
                return current_pos;
            }

            current_pos = current_pos + 1;
        }

        self.0.len() - 1
    }

    /// Returns the bounds of up to `max_lines` before the line pointed to by `position`.
    ///
    /// `position` here is assumed to be the index of the first character of the line
    ///  following the lines to be obtained.
    ///
    /// The return value is a vector of tuples denoting (first_char_index, length) for each line.
    ///
    pub fn find_lines_before(&self, position: usize, max_lines: usize) -> Vec<(usize, usize)> {
        let mut lines = Vec::with_capacity(max_lines);
        let mut last_line_start = position;

        while last_line_start > 0 && (max_lines - lines.len()) > 0 {
            let new_start = self.find_line_start(last_line_start - 1);
            lines.push((new_start, last_line_start - new_start));
            last_line_start = new_start;
        }

        // need to reverse since we read the lines backwards
        lines.reverse();
        lines
    }

    /// Like [CharHelper::find_lines_before], this method returns the bounds of up to `max_lines`
    /// before the line pointed to by `position`. Unlike [CharHelper::find_lines_before], though, this
    /// method also returns the line containing `position` as the last element in the vec.
    ///
    /// `position` can be any index between the start and end of the line.
    pub fn find_line_and_context(&self, position: usize, max_lines: usize) -> Vec<(usize, usize)> {
        let this_start = self.find_line_start(position);
        let this_end = self.find_line_end(position);

        let mut lines = self.find_lines_before(this_start, max_lines);
        lines.push((this_start, this_end - this_start + 1));
        lines
    }

    /// Turns an index into a (line, col) pair.
    pub fn position_to_line_col(&self, position: usize) -> (usize, usize) {
        let mut current_pos: usize = 0;
        let mut current_line: usize = 1;
        let mut last_line_start: usize = 0;

        while current_pos < position {
            if self.0[current_pos] == NEW_LINE {
                current_line = current_line + 1;
                last_line_start = current_pos + 1;
            }

            current_pos = current_pos + 1;
        }

        (current_line, current_pos - last_line_start + 1)
    }
}

/// Allows building a string part by part.
pub struct StringBuilder {
    parts: Vec<String>,
    total_length: usize,
}

impl StringBuilder {
    pub fn new() -> StringBuilder {
        StringBuilder {
            parts: Vec::new(),
            total_length: 0,
        }
    }

    pub fn append(&mut self, str: &impl ToString) {
        let str = str.to_string();
        let len = str.len();
        self.parts.push(str);
        self.total_length = self.total_length + len;
    }

    pub fn append_char(&mut self, c: char) {
        self.parts.push(c.to_string());
        self.total_length = self.total_length + 1;
    }
}

impl ToString for StringBuilder {
    fn to_string(&self) -> String {
        let mut str = String::with_capacity(self.total_length);
        for p in &self.parts {
            str.push_str(p)
        }
        str
    }
}

/// Counts the number of tab characters that appear in the string before `end_index`.
pub fn count_tabs_before(str: &String, end_index: usize) -> usize {
    if end_index == 0 {
        return 0;
    }

    str.chars()
        .take(end_index - 1)
        .filter(|c| c == &TAB)
        .count()
}
