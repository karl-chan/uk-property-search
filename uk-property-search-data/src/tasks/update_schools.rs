use crate::lib::{
    school::{Rating, School},
    util::globals::Globals,
};
use anyhow::Result;
use itertools::multizip;
use mongodb::bson::doc;
use polars::{io::SerReader, prelude::CsvReader};

pub async fn update_schools(globals: &Globals) -> Result<()> {
    let schools_df =
        CsvReader::from_path("assets/2020-2021_england_school_information.csv")?.finish()?;
    let postcodes_df = CsvReader::from_path("assets/ukpostcodes.csv")?.finish()?;
    let merged_df = schools_df.inner_join(&postcodes_df, ["POSTCODE"], ["pcds"])?;

    let ids = merged_df.column("URN")?.i64()?;
    let names = merged_df.column("SCHNAME")?.utf8()?;
    let postcodes = merged_df.column("POSTCODE")?.utf8()?;
    let longitudes = merged_df.column("long")?.f64()?;
    let latitudes = merged_df.column("lat")?.f64()?;
    let rating_strings = merged_df.column("OFSTEDRATING")?.utf8()?;
    let inspection_dates = merged_df.column("OFSTEDLASTINSP")?.utf8()?;

    fn parse_rating_string(rating_string: Option<&str>) -> Rating {
        rating_string.map_or(Rating::Unknown, |s| match s {
            "Outstanding" => Rating::Outstanding,
            "Good" => Rating::Good,
            "Requires improvement" => Rating::RequiresImprovement,
            "Special Measures" | "Serious Weaknesses" | "Inadequate" => Rating::Inadequate,
            _ => Rating::Unknown,
        })
    }

    fn parse_inspection_date(date_string: Option<&str>) -> Option<i64> {
        date_string.map(|s| {
            chrono::NaiveDate::parse_from_str(s, "%d-%m-%Y")
                .unwrap()
                .and_hms(0, 0, 0)
                .timestamp_millis()
        })
    }

    let schools: Vec<School> = multizip((
        ids,
        names,
        postcodes,
        longitudes,
        latitudes,
        rating_strings,
        inspection_dates,
    ))
    .map(
        |(id, name, postcode, longitude, latitude, rating_string, inspection_date)| School {
            id: id.unwrap(),
            name: name.unwrap().into(),
            postcode: postcode.unwrap().into(),
            coordinates: (longitude.unwrap(), latitude.unwrap()),
            rating: parse_rating_string(rating_string) as u8,
            inspection_date_ms: parse_inspection_date(inspection_date),
        },
    )
    .collect();

    let mut session = globals.db.client.start_session(None).await?;
    session.start_transaction(None).await?;

    globals
        .db
        .schools()
        .delete_many_with_session(doc! {}, None, &mut session)
        .await?;
    globals
        .db
        .schools()
        .insert_many_with_session(schools, None, &mut session)
        .await?;
    session.commit_transaction().await?;

    Ok(())
}
