# Changelog

## [Unreleased]

### Fixed
- Fixed "Empty field at position 3 in log entry" error when JJ changes have empty descriptions
- Improved error handling for JJ log parsing to handle missing or empty fields gracefully
- Enhanced JJ log template to use `coalesce()` function for providing default values for empty fields
- Made the parser more robust by only requiring critical fields (change_id and commit_id) to be non-empty

### Technical Changes
- Updated JJ log template to use `coalesce(description, "(no description)")` and `coalesce(author.name(), "(unknown)")`
- Simplified parsing logic to handle empty descriptions and authors gracefully
- Added better error logging for debugging malformed log entries
