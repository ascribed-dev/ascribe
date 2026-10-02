// Package flags keeps flag values in memory.
package flags

import (
	"encoding/json"
	"net/http"
	"strings"
	"sync"
)

// Store holds the current value of each flag.
type Store struct {
	mu     sync.RWMutex
	values map[string]bool
}

// NewStore returns an empty store.
func NewStore() *Store {
	return &Store{values: map[string]bool{}}
}

// Set sets a flag's value.
func (s *Store) Set(key string, enabled bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.values[key] = enabled
}

// Handle answers GET /v2/flags/<key> with the flag's value.
func (s *Store) Handle(w http.ResponseWriter, r *http.Request) {
	key := strings.TrimPrefix(r.URL.Path, "/v2/flags/")
	s.mu.RLock()
	enabled, ok := s.values[key]
	s.mu.RUnlock()
	if !ok {
		http.NotFound(w, r)
		return
	}
	_ = json.NewEncoder(w).Encode(map[string]bool{"enabled": enabled})
}
