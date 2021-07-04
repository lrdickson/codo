package utils

import (
	"bytes"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/exec"
	"os/user"
	"path/filepath"

	// 3rd party
	"gopkg.in/yaml.v2"

	// Internal
	"codo/internal/config"
)

func BuildImage(imagesFolder string, imageName string) {
	// Get the image config
	imageConfig := config.GetImageConfig(imageName)
	passGui := imageConfig[config.PassGUI]
	log.Println(passGui)

	// Read the input docker file
	imageFolder := config.GetImageFolder(imageName)
	inputDockerfile := filepath.Join(imageFolder, "Dockerfile")
	inputDockerfileText, err := ioutil.ReadFile(inputDockerfile)
	if err != nil {
		log.Printf("failed to read %v Dockerfile: %v\n", imageName, err)
		return
	}

	// Create the output Dockerfile
	outputDockerfileText := inputDockerfileText
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

	// Get the image working directory
	command := exec.Command("sudo", "docker", "run",  "--rm", fullImageName, "pwd")
	var commandOutput bytes.Buffer
	command.Stdout = &commandOutput
	err = command.Run()
	if err != nil {
		log.Printf("failed to get %v working directory: %v\n", imageName, err)
		return
	}
	imageWorkingDir := commandOutput.String()
	log.Println(imageWorkingDir)

	// Record the working directory
	storageDir := GetStorageDir(imageName)
	stateFile := filepath.Join(storageDir, "state.yaml")
	imageState := make(map[interface{}]interface{})
	imageState["working-dir"] = imageWorkingDir
	imageStateData, err := yaml.Marshal(&imageState)
	if err != nil {
		log.Printf("failed to create state data for %v: %v\n", imageName, err)
		return
	}
	err = os.WriteFile(stateFile, imageStateData, 0666)
	if err != nil {
		log.Printf("failed to write image state data for %v: %v\n", imageName, err)
		return
	}
}

func GetFullImageName(imageName string) string {
	username := GetUsername()
	return fmt.Sprintf("codo-%v-%v", username, imageName)
}



func GetStorageDir(imageName string) string {
	// Get the storage folder
	home, err := os.UserHomeDir()
	if err != nil {
		log.Fatalf("Unable to get home directory: %v", err)
	}
	storageDir := filepath.Join(home, "codo", imageName)

	// Make sure that the storage folder exists
	err = os.MkdirAll(storageDir, 0755)
	if err != nil {
		log.Fatalf("failed to create storage directory for %v: %v\n", imageName, err)
	}
	return storageDir
}


func GetUsername() string {
	user, err := user.Current()
	if err != nil {
		log.Fatalf("failed to get current user: %v\n", err)
	}
	return user.Username
}
