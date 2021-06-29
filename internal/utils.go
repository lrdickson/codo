package internal

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
)

func BuildImage(imagesFolder string, imageName string) {
	// Get the image config
	imageConfig := GetImageConfig(imageName)
	passGui := imageConfig["pass-gui"]
	log.Println(passGui)

	// Read the input docker file
	imageFolder := GetImageFolder(imageName)
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

func GetImageConfig(imageName string) map[interface{}]interface{} {
	imageConfig := make(map[interface{}]interface{})
	imageConfig["pass-gui"] = true
	imageConfig["attach-pwd"] = true

	// Get the contents of the image config file
	imageFolder := GetImageFolder(imageName)
	imageConfigFile := filepath.Join(imageFolder, "config.yaml")
	imageConfigContent, err := ioutil.ReadFile(imageConfigFile)
	if err != nil {
		log.Printf("failed to read %v config: %v\n", imageName, err)
		return imageConfig
	}

	// Parse the contents
	err = yaml.Unmarshal(imageConfigContent, &imageConfig)
	if err != nil {
		log.Printf("failed to parse %v config: %v\n", imageName, err)
		return imageConfig
	}
	return imageConfig
}

func GetImageFolder(imageName string) string {
	imagesFolder := GetImagesFolder()
	return filepath.Join(imagesFolder, imageName)
}

func GetImagesFolder() string {
	// Get the config folder
	configFolder := GetConfigDir()

	// Build each of the images
	return filepath.Join(configFolder, "images")
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
