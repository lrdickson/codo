package internal

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/exec"
	"os/user"
	"path/filepath"

	// 3rd party
	"gopkg.in/yaml.v2"
)

func BuildImage(imagesFolder string, imageName string) {
	// Get the contents of the image config file
	imageFolder := filepath.Join(imagesFolder, imageName)
	imageConfigFile := filepath.Join(imageFolder, "config.yaml")
	imageConfigContent, err := ioutil.ReadFile(imageConfigFile)
	if err != nil {
		log.Printf("failed to read %v config: %v\n", imageName, err)
		return
	}

	// Parse the contents
	imageConfig := make(map[interface{}]interface{})
	err = yaml.Unmarshal(imageConfigContent, &imageConfig)
	if err != nil {
		log.Printf("failed to read %v config: %v\n", imageName, err)
		return
	}
	passGui, configHasPassGui := imageConfig["pass-gui"]
	if ! configHasPassGui {
		passGui = false
	}
	log.Println(passGui)

	// Read the input docker file
	inputDockerfile := filepath.Join(imageFolder, "Dockerfile")
	inputDockerfileText, err := ioutil.ReadFile(inputDockerfile)
	if err != nil {
		log.Printf("failed to read %v Dockerfile: %v\n", imageName, err)
		return
	}
	outputDockerfileText := inputDockerfileText

	// Create the output Dockerfile
	buildDir := filepath.Join("/","tmp", "codo", imageName)
	err = os.MkdirAll(buildDir, 0755)
	if err != nil {
		log.Printf("failed to create Dockerfile directory for %v: %v\n", imageName, err)
		return
	}
	outputDockerfile := filepath.Join(buildDir, "Dockerfile")
	err = os.WriteFile(outputDockerfile, []byte(outputDockerfileText), 0666)
	if err != nil {
		log.Printf("failed to write Dockerfile for %v: %v\n", imageName, err)
		return
	}

	// Build the image
	fullImageName := GetFullImageName(imageName)
	buildCommand := exec.Command("sudo", "docker", "build", "-t", fullImageName, buildDir)
	err = buildCommand.Run()
	if err != nil {
		log.Printf("failed to build image for %v: %v\n", imageName, err)
		return
	}
}

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
