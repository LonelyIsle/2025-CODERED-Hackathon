package db

import (
	"context"
	"errors"
)

type User struct {
	ID       int
	Username string
	Password string
	Role     string
}

func GetUserByUsername(ctx context.Context, username string) (*User, error) {
	row := Pool.QueryRow(ctx, "SELECT id, username, password, role FROM users WHERE username=$1", username)
	u := &User{}
	if err := row.Scan(&u.ID, &u.Username, &u.Password, &u.Role); err != nil {
		return nil, errors.New("user not found")
	}
	return u, nil
}