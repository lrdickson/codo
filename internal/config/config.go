package config

import (
	"io/ioutil"
	"log"
	"os"
	"path/filepath"

	// 3rd party
	"gopkg.in/yaml.v2"
)

const BindWorkingDir string = "bind-working-dir"
const PassGUI string = "pass-gui"

func GetConfigDir() string {
	// Get the config folder
	home, err := os.UserHomeDir()
	if err != nil {
		log.Fatalf("Unable to get home directory: %v", err)
	}
	return filepath.Join(home, ".config", "codo")
}


func GetImageConfig(imageName string) map[interface{}]interface{} {
	imageConfig := make(map[interface{}]interface{})
	imageConfig[PassGUI] = true
	imageConfig[BindWorkingDir] = true

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


