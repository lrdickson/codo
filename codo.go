package main

import (
	// Standard library
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/exec"
	"path/filepath"

	// 3rd party libraries
	"gopkg.in/yaml.v2"

	// Internal
	"codo/internal/config"
	"codo/internal/utils"
)

func main() {
	// Determine if there is any command to run
	if len(os.Args) < 2 {
		return
	}

	// Get the config folder
	configFolder := config.GetConfigDir()

	// Get the contents of the image config file
	codoConfigFile := filepath.Join(configFolder, "config.yaml")
	codoConfigContent, err := ioutil.ReadFile(codoConfigFile)
	if err != nil {
		log.Fatalf("failed to read config: %v\n", err)
	}

	// Parse the contents
	codoConfig := make(map[interface{}]interface{})
	err = yaml.Unmarshal(codoConfigContent, &codoConfig)
	if err != nil {
		log.Fatalf("failed to parse config: %v\n", err)
	}

	// Get the default image name
	defaultImage, configHasDefaultImage := codoConfig["default-image"]
	if ! configHasDefaultImage {
		log.Fatalf("no default image found in %v\n", configFolder)
	}
	defaultImageName := defaultImage.(string)
	imageName := defaultImageName

	// Build the command to run
	commandContents := []string{"sudo", "docker", "run", "-ti", "--rm"}

	// Bind the working directory
	imageConfig := config.GetImageConfig(imageName)
	bindWorkdingDir := imageConfig[config.BindWorkingDir].(bool)
	if bindWorkdingDir {
		workingDirectory, err := os.Getwd()
		if err != nil {
			log.Fatalf("failed to get the host working directory: %v\n", err)
		}
		bindWDParam := fmt.Sprintf("%v:/codo", workingDirectory)
		commandContents = append(commandContents, "-v", bindWDParam, "-w", "/codo")
	}

	// Add the image name
	fullImageName := utils.GetFullImageName(imageName)
	commandContents = append(commandContents, fullImageName)

	// Add the argument to be run in the container
	commandContents = append(commandContents, os.Args[1:]...)

	// Run the command in the docker container
	//log.Printf("Running %v\n", commandContents)
	command := exec.Command(commandContents[0], commandContents[1:]...)
	command.Stdin = os.Stdin
	command.Stdout = os.Stdout
	command.Stderr = os.Stderr
	err = command.Run()
	if err != nil {
		log.Fatalf("failed to run command %v: %v\n", commandContents, err)
	}
}
