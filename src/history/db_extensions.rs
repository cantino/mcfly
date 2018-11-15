extern crate rusqlite;
extern crate regex;

use rusqlite::{Connection};
use weights::Weights;

pub fn add_db_functions(db: &Connection) {
    let weights = Weights::default();
    db.create_scalar_function("nn_rank", 10, true, move |ctx| {
        let length_factor = ctx.get::<f64>(0)?;
        let age_factor = ctx.get::<f64>(1)?;
        let exit_factor = ctx.get::<f64>(2)?;
        let recent_failure_factor = ctx.get::<f64>(3)?;
        let dir_factor = ctx.get::<f64>(4)?;
        let selected_dir_factor = ctx.get::<f64>(5)?;
        let overlap_factor = ctx.get::<f64>(6)?;
        let immediate_overlap_factor = ctx.get::<f64>(7)?;
        let selected_occurrences_factor = ctx.get::<f64>(8)?;
        let occurrences_factor = ctx.get::<f64>(9)?;

        let result: f64 = weights.offset +
          length_factor * weights.length +
          age_factor * weights.age +
          exit_factor * weights.exit +
          recent_failure_factor * weights.recent_failure +
          dir_factor * weights.dir +
          selected_dir_factor * weights.selected_dir +
          overlap_factor * weights.overlap +
          immediate_overlap_factor * weights.immediate_overlap +
          selected_occurrences_factor * weights.selected_occurrences +
          occurrences_factor * weights.occurrences;

        Ok(result)
    }).expect("Successful create_scalar_function");
}
