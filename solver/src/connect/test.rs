use super::*;

#[test]
fn path_stats() {
    assert!(PathStats::new(1, 0) > PathStats::new(4, 0));
    assert!(PathStats::new(1, 0) == PathStats::new(1, 0));
    assert!(PathStats::new(2, 0) < PathStats::new(1, 0));
    
    assert!(PathStats::new(1, 3) > PathStats::new(1, 1));
    assert!(PathStats::new(1, 2) == PathStats::new(1, 2));
    assert!(PathStats::new(1, 1) < PathStats::new(1, 3));
}

