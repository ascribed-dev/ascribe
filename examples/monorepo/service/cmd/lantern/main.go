// The Lantern service: serves flag values to SDKs. A stand-in for the code in
// a repository whose documentation lives beside it.
package main

import (
	"log"
	"net/http"

	"example.com/lantern/internal/flags"
)

func main() {
	store := flags.NewStore()
	store.Set("new-checkout", false)

	http.HandleFunc("/v2/flags/", store.Handle)
	log.Println("listening on :8080")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
