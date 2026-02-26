use std::{
    collections::BTreeMap,
    io::BufReader,
    io::Write,
    path::{Path, PathBuf},
};

use is_terminal::IsTerminal;
use yansi::Paint;

use crate::{
    cli::{Selector, ShowFormat},
    config::Configuration,
    error::{Error, FragmentError},
    fragment::Fragment,
};

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct Show {
    format: Option<crate::cli::ShowFormat>,
    selector: Option<Selector>,
}

impl crate::command::Command for Show {
    fn execute(
        self,
        workdir: &Path,
        config: &Configuration,
    ) -> Result<Option<std::process::ExitCode>, Error> {
        let pathes =
            crate::selector::SelectorExecutor::new(self.selector.as_ref()).run(workdir, config)?;

        tracing::trace!("Looking at: {pathes:?}");
        let fragments = pathes.into_iter().map(|path| {
            std::fs::OpenOptions::new()
                .read(true)
                .create(false)
                .write(false)
                .open(&path)
                .map_err(FragmentError::from)
                .map(BufReader::new)
                .and_then(|mut reader| {
                    Fragment::from_reader(&mut reader).map(|f| (path.to_path_buf(), f))
                })
                .map_err(|e| Error::Fragment(e, path.to_path_buf()))
        });

        match self.format {
            None | Some(ShowFormat::Text) => pretty_print(fragments)?,
            Some(ShowFormat::Json) => json_print(fragments)?,
        }

        Ok(None)
    }
}

fn pretty_print(
    mut iter: impl Iterator<Item = Result<(PathBuf, Fragment), Error>>,
) -> Result<(), Error> {
    let out = std::io::stdout();
    let mut output = out.lock();

    let is_terminal = std::io::stdout().is_terminal();
    if !is_terminal {
        yansi::disable()
    }

    iter.try_for_each(|fragment| {
        let (path, fragment) = fragment?;
        writeln!(output, "{}", Paint::new(path.display()).bold())?;
        fragment.header().iter().try_for_each(|(key, value)| {
            writeln!(
                output,
                "{key}: {value}",
                key = Paint::new(key).italic(),
                value = value.display()
            )?;
            Ok(()) as Result<(), Error>
        })?;

        writeln!(output, "{text}", text = fragment.text())?;
        writeln!(output)?;
        Ok(())
    })
}

fn json_print(iter: impl Iterator<Item = Result<(PathBuf, Fragment), Error>>) -> Result<(), Error> {
    let v = iter.collect::<Result<BTreeMap<PathBuf, Fragment>, _>>()?;
    let out = std::io::stdout();
    let output = out.lock();
    serde_json::to_writer(output, &v).map_err(Error::from)
}
