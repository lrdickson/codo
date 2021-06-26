package main

import (
	// Standard library
	"io/ioutil"
	"log"
	"os"
	"os/exec"
	"path/filepath"

	// 3rd party libraries
	"gopkg.in/yaml.v2"

	// Internal
	"codo/internal"
)

func main() {
	// Get the config folder
	configFolder := internal.GetConfigDir()

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
		log.Printf("failed to parse config: %v\n", err)
		return
	}

	// Get the default image name
	defaultImage, configHasDefaultImage := codoConfig["default-image"]
	if ! configHasDefaultImage {
		log.Fatalf("no default image found in %v\n", configFolder)
	}

	// Run the command in the docker container
	fullImageName := internal.GetFullImageName(defaultImage.(string))
	commandContents :=	[]string{"sudo", "docker", "run", "--rm", fullImageName}
	commandContents = append(commandContents, os.Args[1:]...)
	command := exec.Command(commandContents[0], commandContents[1:]...)
	err = command.Run()
	if err != nil {
		log.Fatalf("failed to run command %v: %v\n", commandContents, err)
	}
}
