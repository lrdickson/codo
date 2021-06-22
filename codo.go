package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"

	//"gopkg.in/yaml.v2"
)

type ImageConfig struct {
	BaseImage string
}

func BuildImage(imagesFolder string, imageName string) {
	// Get the contents of the image config file
	imageFolder := filepath.Join(imagesFolder, imageName)
	imageConfigFile := filepath.Join(imageFolder, "config.yaml")
	imageConfigContent, err := ioutil.ReadFile(imageConfigFile)
	if err != nil {
		log.Printf("failed to read %v config: %v", imageName, err)
		return
	}
	fmt.Println(string(imageConfigContent))

	//
}


func main() {
	// Get the config folder
	home, err := os.UserHomeDir()
	if err != nil {
		log.Fatalf("error: %v", err)
	}
	configFolder := filepath.Join(home, ".config", "codo")

	// Build each of the images
	imagesFolder := filepath.Join(configFolder, "images")
	imagesFolderContents, err := ioutil.ReadDir(imagesFolder)
	if err != nil {
		log.Fatalf("error: %v", err)
	}
	for _, content := range imagesFolderContents {
		if content.IsDir() {
			BuildImage(imagesFolder, content.Name())
		}
	}
}
