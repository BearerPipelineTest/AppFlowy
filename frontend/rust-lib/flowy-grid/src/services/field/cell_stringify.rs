use crate::services::field::*;
use crate::services::util::*;
use flowy_error::FlowyError;
use flowy_grid_data_model::entities::{AnyData, Field, FieldType};

pub trait StringifyCellData {
    fn str_from_cell_data(&self, data: AnyData) -> String;
    fn str_to_cell_data(&self, s: &str) -> Result<AnyData, FlowyError>;
}

#[allow(dead_code)]
pub fn stringify_serialize(field: &Field, s: &str) -> Result<AnyData, FlowyError> {
    match field.field_type {
        FieldType::RichText => RichTextDescription::from(field).str_to_cell_data(s),
        FieldType::Number => NumberDescription::from(field).str_to_cell_data(s),
        FieldType::DateTime => DateDescription::from(field).str_to_cell_data(s),
        FieldType::SingleSelect => SingleSelectDescription::from(field).str_to_cell_data(s),
        FieldType::MultiSelect => MultiSelectDescription::from(field).str_to_cell_data(s),
        FieldType::Checkbox => CheckboxDescription::from(field).str_to_cell_data(s),
    }
}

pub(crate) fn stringify_deserialize(data: AnyData, field: &Field) -> Result<String, FlowyError> {
    let _ = check_type_id(&data, field)?;
    let s = match field.field_type {
        FieldType::RichText => RichTextDescription::from(field).str_from_cell_data(data),
        FieldType::Number => NumberDescription::from(field).str_from_cell_data(data),
        FieldType::DateTime => DateDescription::from(field).str_from_cell_data(data),
        FieldType::SingleSelect => SingleSelectDescription::from(field).str_from_cell_data(data),
        FieldType::MultiSelect => MultiSelectDescription::from(field).str_from_cell_data(data),
        FieldType::Checkbox => CheckboxDescription::from(field).str_from_cell_data(data),
    };
    Ok(s)
}