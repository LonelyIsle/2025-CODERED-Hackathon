package util

// ChunkRunes splits s into ~maxLen rune chunks.
// (Fast + simple; good enough for HTML/text you scraped.)
func ChunkRunes(s string, maxLen int) []string {
	if maxLen <= 0 {
		maxLen = 1500 // ~300–400 tokens
	}
	rs := []rune(s)
	n := len(rs)
	out := make([]string, 0, (n/maxLen)+1)
	for i := 0; i < n; i += maxLen {
		j := i + maxLen
		if j > n { j = n }
		out = append(out, string(rs[i:j]))
	}
	return out
}
