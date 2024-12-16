use clap::Parser;
use colored::*;
use csv::{ReaderBuilder, Writer};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;
use unicode_segmentation::UnicodeSegmentation;

/// Contact struct represents a single contact record
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Contact {
    #[serde(rename = "contactID")]
    id: i32,
    name: String,
    #[serde(rename = "name1")]
    last_name: String,
    email: String,
    #[serde(rename = "postalZip")]
    zip_code: String,
    address: String,
}

/// SimilarityScore represents the matching score between two contacts
#[derive(Debug, Serialize)]
struct SimilarityScore {
    contact_id1: i32,
    contact_id2: i32,
    score: i32,
}

/// CLI argument parser struct using clap
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, value_parser)]
    file: PathBuf,

    #[clap(short, long, value_parser)]
    output: PathBuf,

    // Threshold for similarity score, default is 800
    #[clap(short = 't', long, value_parser = clap::value_parser!(i32).range(0..=1000), default_value = "800")]
    threshold: i32,
}

/// Calculates the Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<&str> = s1.graphemes(true).collect();
    let s2_chars: Vec<&str> = s2.graphemes(true).collect();

    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    // Initialize matrix for dynamic programming
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column using iterators
    matrix
        .iter_mut()
        .enumerate()
        .take(len1 + 1)
        .for_each(|(i, row)| row[0] = i);

    matrix[0]
        .iter_mut()
        .enumerate()
        .take(len2 + 1)
        .for_each(|(j, cell)| *cell = j);

    // Fill the matrix using dynamic programming
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}

/// Calculates similarity score between two strings using Levenshtein distance
/// Time complexity: O(mn) where m and n are the lengths of input strings
/// Space complexity: O(mn) due to levenshtein_distance function
fn calculate_similarity(s1: &str, s2: &str) -> f64 {
    if s1.is_empty() && s2.is_empty() {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }
    let distance = levenshtein_distance(s1, s2);
    let max_len = std::cmp::max(s1.chars().count(), s2.chars().count());
    1.0 - (distance as f64 / max_len as f64)
}

/// Calculates match score between two contacts
fn calculate_match_score(contact1: &Contact, contact2: &Contact) -> i32 {
    let mut score = 0.0;

    // Weight multipliers for different fields
    score += calculate_similarity(&contact1.name, &contact2.name) * 2.0;
    score += calculate_similarity(&contact1.last_name, &contact2.last_name) * 2.0;
    score += calculate_similarity(&contact1.email, &contact2.email) * 3.0;
    score += calculate_similarity(&contact1.zip_code, &contact2.zip_code) * 2.0;
    score += calculate_similarity(&contact1.address, &contact2.address) * 2.0;

    // Normalize to 0-1000 scale
    ((score * 1000.0) / 11.0) as i32
}

/// Reads contacts from CSV file
fn read_contacts_from_csv(path: &PathBuf) -> Result<Vec<Contact>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(path)?;

    let contacts: Result<Vec<Contact>, _> = rdr.deserialize().collect();
    contacts.map_err(|e| e.into())
}

/// Writes similarity scores to CSV file
fn write_scores_to_csv(path: &PathBuf, scores: &[SimilarityScore]) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;

    // Write headers
    wtr.write_record(["ContactID1", "ContactID2", "SimilarityScore"])?;

    // Write data
    for score in scores {
        wtr.write_record(&[
            score.contact_id1.to_string(),
            score.contact_id2.to_string(),
            score.score.to_string(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

/// Calculates similarity scores for all contact pairs in parallel
fn calculate_similarity_scores(contacts: &[Contact], threshold: i32) -> Vec<SimilarityScore> {
    // Generate all possible pairs of contacts
    let pairs: Vec<(&Contact, &Contact)> = contacts
        .iter()
        .enumerate()
        .flat_map(|(i, c1)| contacts[i + 1..].iter().map(move |c2| (c1, c2)))
        .collect();

    // Calculate scores in parallel using rayon
    pairs
        .par_iter() // Use rayon for parallel processing
        .filter_map(|&(contact1, contact2)| {
            let score = calculate_match_score(contact1, contact2);
            if score >= threshold {
                Some(SimilarityScore {
                    contact_id1: contact1.id,
                    contact_id2: contact2.id,
                    score,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Main function orchestrating the entire process
/// Overall Time Complexity: O(N²L) where:
/// - N is the number of contacts
/// - L is the maximum length of any field in the contacts
///
/// Overall Space Complexity: O(N² + RC) where:
/// - N is the number of contacts (for storing all pair combinations)
/// - R is number of rows in input file
/// - C is average size of contact data
///
/// The bottleneck operations are:
/// 1. Calculating similarity scores: O(N²L)
/// 2. Storing all pair combinations: O(N²)
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Read contacts from input CSV file
    let contacts = read_contacts_from_csv(&args.file)?;

    // Calculate similarity scores with threshold
    let scores = calculate_similarity_scores(&contacts, args.threshold);

    // Write results to output CSV file
    write_scores_to_csv(&args.output, &scores)?;

    println!(
        "{} {} {} {}. {} {}",
        "Found".bright_white(),
        scores.len().to_string().bright_green().bold(),
        "matches above threshold".bright_white(),
        args.threshold.to_string().yellow().bold(),
        "Results written to".bright_white(),
        args.output.to_string_lossy().bright_blue().underline()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Utility function to create test contacts
    fn create_test_contact(
        id: i32,
        name: &str,
        last_name: &str,
        email: &str,
        zip_code: &str,
        address: &str,
    ) -> Contact {
        Contact {
            id,
            name: name.to_string(),
            last_name: last_name.to_string(),
            email: email.to_string(),
            zip_code: zip_code.to_string(),
            address: address.to_string(),
        }
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
    }
    #[test]
    fn test_levenshtein_distance_edge_cases() {
        // Empty strings
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);

        // Unicode characters
        assert_eq!(levenshtein_distance("café", "cafe"), 1);
        assert_eq!(levenshtein_distance("über", "uber"), 1);
        assert_eq!(levenshtein_distance("🌟", "⭐"), 1);

        // Very different lengths
        assert_eq!(levenshtein_distance("a", "abcdef"), 5);
        assert_eq!(levenshtein_distance("abcdef", "a"), 5);
    }
    #[test]
    fn test_calculate_similarity_edge_cases() {
        // Empty strings
        assert_eq!(calculate_similarity("", ""), 1.0);
        assert_eq!(calculate_similarity("abc", ""), 0.0);
        assert_eq!(calculate_similarity("", "abc"), 0.0);

        // Very similar strings
        assert!(calculate_similarity("color", "colour") > 0.8);

        // Very different strings
        assert!(calculate_similarity("completely", "different") < 0.3);
    }

    #[test]
    fn test_calculate_similarity() {
        assert_eq!(calculate_similarity("", ""), 1.0);
        assert_eq!(calculate_similarity("abc", "abc"), 1.0);
        assert!(calculate_similarity("kitten", "sitting") < 1.0);
    }

    #[test]
    fn test_calculate_match_score() {
        let contact1 =
            create_test_contact(1, "John", "Doe", "jhon@example.com", "12345", "123 Main St");
        let mut contact2 = contact1.clone();
        contact2.id = 2;

        let score = calculate_match_score(&contact1, &contact2);
        assert_eq!(score, 1000); // Perfect match should give maximum score
    }
    #[test]
    fn test_calculate_match_score_similar_contacts() {
        let contact1 =
            create_test_contact(1, "John", "Doe", "john@example.com", "12345", "123 Main St");
        let contact2 = create_test_contact(
            2,
            "Jon",
            "Doe",
            "john@example.com",
            "12345",
            "123 Main Street",
        );

        let score = calculate_match_score(&contact1, &contact2);
        assert_eq!(score, 906); // Similar contacts should have high score
    }

    #[test]
    fn test_calculate_match_score_different_contacts() {
        let contact1 =
            create_test_contact(1, "John", "Doe", "john@example.com", "12345", "123 Main St");
        let contact2 = create_test_contact(
            2,
            "Jane",
            "Smith",
            "jane@example.com",
            "54321",
            "456 Oak Ave",
        );

        let score = calculate_match_score(&contact1, &contact2);
        assert_eq!(score, 336); // Different contacts should have lower score
    }

    #[test]
    fn test_calculate_match_score_edge_cases() {
        // Empty fields
        let empty_contact1 = create_test_contact(1, "", "", "", "", "");
        let empty_contact2 = create_test_contact(2, "", "", "", "", "");
        assert_eq!(
            calculate_match_score(&empty_contact1, &empty_contact2),
            1000
        );

        // One empty contact
        let normal_contact =
            create_test_contact(1, "John", "Doe", "john@example.com", "", "123 Main St");
        let score = calculate_match_score(&normal_contact, &empty_contact1);
        assert_eq!(score, 181);

        // Unicode characters
        let unicode_contact1 = create_test_contact(
            1,
            "José",
            "García",
            "jose@example.com",
            "12345",
            "123 Café St",
        );
        let unicode_contact2 = create_test_contact(
            2,
            "Jose",
            "Garcia",
            "jose@example.com",
            "12345",
            "123 Cafe St",
        );
        let unicode_score = calculate_match_score(&unicode_contact1, &unicode_contact2);
        println!("Unicode score: {}", unicode_score);
        assert_eq!(unicode_score, 907); // Should still be high despite accent differences
    }
}
