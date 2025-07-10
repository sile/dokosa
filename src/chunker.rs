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

impl<'text, 'raw, T> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Chunk<T>
where
    T: TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>,
{
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Chunk {
            line: value.to_member("line")?.required()?.try_into()?,
            data: value.to_member("data")?.required()?.try_into()?,
        })
    }
}
