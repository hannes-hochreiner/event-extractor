export def test [] {
  ^cargo test
}

export def build [] {
  test
  ^cargo build
}