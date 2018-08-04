extern crate serde_yaml;
use spec::*;

#[derive(Serialize, Deserialize)]
pub struct ProjectYaml {
    project: serde_yaml::Value,

    #[serde(default = "Vec::new")]
    components: Vec<serde_yaml::Value>,

    #[serde(default = "Vec::new")]
    effects: Vec<serde_yaml::Value>,
}

pub trait ProjectLoader : HaveProject + HaveComponentRepository + HaveEffectRepository {
    fn to_yaml(&self) -> Result<serde_yaml::Value, serde_yaml::Error> {
        serde_yaml::to_value(ProjectYaml {
            project: serde_yaml::to_value(self.project())?,
            components: self.component_repo().list().iter().map(|v| serde_yaml::to_value(v).unwrap()).collect(),
            effects: self.effect_repo().list().iter().map(|v| serde_yaml::to_value(v).unwrap()).collect(),
        })
    }

    fn from_yaml(&mut self, value: serde_yaml::Value) -> Result<(), serde_yaml::Error> {
        let yaml = serde_yaml::from_value::<ProjectYaml>(value)?;

        {
            let project = serde_yaml::from_value::<Project>(yaml.project)?;
            self.project_mut().layers = project.layers;
            self.project_mut().size = project.size;
            self.project_mut().length = project.length;
        }

        self.component_repo_mut().load_table(
            yaml.components.into_iter().map(|v| serde_yaml::from_value(v).unwrap()).collect()
        );
        self.effect_repo_mut().load_table(
            yaml.effects.into_iter().map(|v| serde_yaml::from_value(v).unwrap()).collect()
        );

        Ok(())
    }
}

