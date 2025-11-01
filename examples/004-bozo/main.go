//go:generate go run github.com/99designs/gqlgen generate

package main

import (
	"net/http"
	"strings"
	"context"

	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/99designs/gqlgen/graphql/handler/extension"
	"github.com/99designs/gqlgen/graphql/handler/lru"
	"github.com/99designs/gqlgen/graphql/handler/transport"
	"github.com/stylus-developers-guild/bobcat-sdk/examples/004-bozo/graph"
	"github.com/vektah/gqlparser/v2/ast"
)

type corsMiddleware struct{ srv *handler.Server }

func (m corsMiddleware) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	w.Header().Set("Access-Control-Allow-Headers", "*")
	ipAddrs := strings.Split(r.Header.Get("X-Forwarded-For"), ",")
	var ipAddr string
	if len(ipAddrs) > 0 {
		ipAddr = ipAddrs[0]
	}
	ctx := context.WithValue(r.Context(), "ip addr", ipAddr)
	m.srv.ServeHTTP(w, r.WithContext(ctx))
}

func main() {
	srv := handler.New(graph.NewExecutableSchema(graph.Config{
		Resolvers: &graph.Resolver{},
	}))
	srv.AddTransport(transport.Options{})
	srv.AddTransport(transport.GET{})
	srv.AddTransport(transport.POST{})
	srv.SetQueryCache(lru.New[*ast.QueryDocument](1000))
	srv.Use(extension.Introspection{})
	srv.Use(extension.AutomaticPersistedQuery{
		Cache: lru.New[string](100),
	})
	http.Handle("/", corsMiddleware{srv})
	panic(http.ListenAndServe(":8080", nil))
}
