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

pub trait ProjectLoader : HaveProject + HaveEffectRepository + HaveComponentRepository {
    fn to_yaml_string(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(&self.to_yaml()?)
    }

    fn from_yaml_string(&mut self, value: &str) -> Result<(), serde_yaml::Error> {
        self.from_yaml(serde_yaml::from_str(value)?)
    }

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
            self.project_mut().size = project.size as (i32,i32);
            self.project_mut().length = project.length;
        }

        self.component_repo_mut().load_table(
            yaml.components.into_iter().map(|v| {
                let entity = serde_yaml::from_value::<Entity<serde_yaml::Value, String>>(v).unwrap();
                let component = <Self as HaveComponentRepository>::new_from_json(
                    serde_yaml::from_value(entity.entity).unwrap()
                );

                Entity {
                    id: entity.id,
                    entity: component,
                }
            }).collect()
        );
        self.effect_repo_mut().load_table(
            yaml.effects.into_iter().map(|v| serde_yaml::from_value(v).unwrap()).collect()
        );

        Ok(())
    }
}

