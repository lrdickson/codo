package internal

import (
	"fmt"
	"log"
	"os"
	"os/user"
	"path/filepath"
)

func GetConfigDir() string {
	// Get the config folder
	home, err := os.UserHomeDir()
	if err != nil {
		log.Fatalf("Unable to get home directory: %v", err)
	}
	return filepath.Join(home, ".config", "codo")
}

func GetFullImageName(imageName string) string {
	username := GetUsername()
	return fmt.Sprintf("codo-%v-%v", username, imageName)
}

func GetUsername() string {
	user, err := user.Current()
	if err != nil {
		log.Fatalf("failed to get current user: %v\n", err)
	}
	return user.Username
}
