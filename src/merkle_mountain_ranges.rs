use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
struct MMR {
    peaks: Vec<String>,
    leaves: Vec<String>,
    bag_size: usize,
}

impl MMR {
    fn new(bag_size: usize) -> Self {
        MMR {
            peaks: Vec::new(),
            leaves: Vec::new(),
            bag_size,
        }
    }

    fn append(&mut self, data: &str) {
        let leaf_hash = hash(data);
        self.leaves.push(leaf_hash.clone());
        
        let mut current_hash = leaf_hash;
        let mut height = 0;
        let mut new_peaks = Vec::new();
        
        while height < self.peaks.len() && !self.peaks[height].is_empty() {
            current_hash = hash(&format!("{}{}", self.peaks[height], current_hash));
            self.peaks[height] = String::new();
            height += 1;
        }
        
        new_peaks.push(current_hash);
        
        for peak in new_peaks {
            if height == self.peaks.len() {
                self.peaks.push(peak);
            } else {
                self.peaks[height] = peak;
            }
            height += 1;
        }
        
        self.bag_peaks();
    }

    fn bag_peaks(&mut self) {
        let mut i = 0;
        while i + self.bag_size <= self.peaks.len() {
            let mut all_non_empty = true;
            let mut bag = String::new();
            for j in 0..self.bag_size {
                if self.peaks[i + j].is_empty() {
                    all_non_empty = false;
                    break;
                }
                bag += &self.peaks[i + j];
            }
            if all_non_empty {
                let bagged_hash = hash(&bag);
                self.peaks[i] = bagged_hash;
                for j in 1..self.bag_size {
                    self.peaks[i + j] = String::new();
                }
            }
            i += 1;
        }
    }

    fn root(&self) -> String {
        let mut current_hash = String::new();
        for peak in self.peaks.iter().rev() {
            if !peak.is_empty() {
                if current_hash.is_empty() {
                    current_hash = peak.clone();
                } else {
                    current_hash = hash(&format!("{}{}", peak, current_hash));
                }
            }
        }
        current_hash
    }
   
}

fn hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bagged_peaks() {
        let mut mmr = MMR::new(2);
        mmr.append("A");
        mmr.append("B");
        mmr.append("C");
        mmr.append("D");
        mmr.append("E");
        mmr.append("F");
        mmr.append("G");
        mmr.append("H");

        let non_empty_peaks = mmr.peaks.iter().filter(|&p| !p.is_empty()).count();
        assert_eq!(non_empty_peaks, 1, "Expected 1 non-empty peak, got {}", non_empty_peaks);

        let root1 = mmr.root();
        mmr.append("I");
        let root2 = mmr.root();
        assert_ne!(root1, root2);
    }

    #[test]
    fn test_multiple_bagging() {
        let mut mmr = MMR::new(3);
        for i in 0..10 {
            mmr.append(&i.to_string());
        }

        let non_empty_peaks = mmr.peaks.iter().filter(|&p| !p.is_empty()).count();
        assert!(non_empty_peaks <= (10 as f64).log2().ceil() as usize, "Num non-empty peaks <= log2(n)");

        let root1 = mmr.root();
        mmr.append("10");
        let root2 = mmr.root();
        assert_ne!(root1, root2, "Root should change after append");
    }
 
}