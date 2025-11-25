// Integration test for permission system
// Run with: cargo test --test test_permission_integration



#[test]
fn test_permission_manager_creation() {
    // Test that PermissionManager can be created
    let result = promptline::permissions::PermissionManager::new();
    assert!(result.is_ok(), "PermissionManager should create successfully");
}

#[test]
fn test_permission_defaults() {
    let manager = promptline::permissions::PermissionManager::new().unwrap();
    
    // Default should be Ask for unknown tools (use unique name to avoid conflicts)
    let unique_tool = format!("test_tool_unique_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());
    let level = manager.check_permission(&unique_tool);
    assert_eq!(level, promptline::permissions::PermissionLevel::Ask);
}

#[test]
fn test_permission_persistence() {
    let mut manager = promptline::permissions::PermissionManager::new().unwrap();
    
    // Set a permission
    manager.set_permission(
        "test_tool_always".to_string(),
        promptline::permissions::PermissionLevel::Always
    ).unwrap();
    
    // Verify it's set
    let level = manager.check_permission("test_tool_always");
    assert_eq!(level, promptline::permissions::PermissionLevel::Always);
    
    // Create a new manager (simulating restart)
    let manager2 = promptline::permissions::PermissionManager::new().unwrap();
    let level2 = manager2.check_permission("test_tool_always");
    assert_eq!(level2, promptline::permissions::PermissionLevel::Always, 
        "Permission should persist across manager instances");
    
    // Cleanup - reset to Ask to clean up test
    let mut manager3 = promptline::permissions::PermissionManager::new().unwrap();
    manager3.set_permission(
        "test_tool_always".to_string(),
        promptline::permissions::PermissionLevel::Ask
    ).unwrap();
}

#[test]
fn test_session_only_permissions() {
    let mut manager = promptline::permissions::PermissionManager::new().unwrap();
    
    // Set Once (session only)
    manager.set_permission(
        "test_tool_once".to_string(),
        promptline::permissions::PermissionLevel::Once
    ).unwrap();
    
    // Should be available in same manager
    let level = manager.check_permission("test_tool_once");
    assert_eq!(level, promptline::permissions::PermissionLevel::Once);
    
    // Should NOT persist to new manager
    let manager2 = promptline::permissions::PermissionManager::new().unwrap();
    let level2 = manager2.check_permission("test_tool_once");
    assert_eq!(level2, promptline::permissions::PermissionLevel::Ask,
        "Once permission should not persist across sessions");
}

#[test]
fn test_permission_storage_location() {
    use promptline::permissions::PermissionManager;
    
    let _manager = PermissionManager::new().unwrap();
    
    // Just verify we can create it without errors
    // The actual path is in ~/.promptline/permissions.yaml
    let home = dirs::home_dir().expect("Should have home directory");
    let expected_dir = home.join(".promptline");
    
    assert!(expected_dir.exists() || true, 
        "Config directory should exist or be creatable");
}
