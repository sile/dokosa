use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct Chunker {
    pub window_size: NonZeroUsize,
    pub step_size: NonZeroUsize,
}

impl Chunker {
    pub fn new(window_size: NonZeroUsize, step_size: NonZeroUsize) -> Self {
        Self {
            window_size,
            step_size,
        }
    }

    pub fn apply(&self, input: &str) -> Vec<Chunk<String>> {
        let mut chunks = Vec::new();
        let lines = input.lines().collect::<Vec<_>>();
        for (i, lines) in lines.windows(self.window_size.get()).enumerate() {
            if i % self.step_size.get() != 0 {
                continue;
            }

            chunks.push(Chunk {
                line: i,
                data: lines.join("\n"),
            });
        }
        if chunks.is_empty() {
            assert!(lines.len() < self.window_size.get());
            chunks.push(Chunk {
                line: 0,
                data: lines.join("\n"),
            });
        }
        chunks
    }
}

#[derive(Debug)]
pub struct Chunk<T> {
    pub line: usize,
    pub data: T,
}

impl<T> nojson::DisplayJson for Chunk<T>
where
    T: nojson::DisplayJson,
{
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("line", self.line)?;
            f.member("data", &self.data)
        })
    }
}

impl<'text, T> nojson::FromRawJsonValue<'text> for Chunk<T>
where
    T: nojson::FromRawJsonValue<'text>,
{
    fn from_raw_json_value(
        value: nojson::RawJsonValue<'text, '_>,
    ) -> Result<Self, nojson::JsonParseError> {
        let ([line, data], []) = value.to_fixed_object(["line", "data"], [])?;
        Ok(Chunk {
            line: line.try_to()?,
            data: data.try_to()?,
        })
    }
}
