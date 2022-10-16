use crate::{
    errors::Error,
    indexes::Index,
    request::{request, Method},
    task_info::TaskInfo,
};
use std::result::Result as StdResult;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Setting<T> {
    Set(T),
    Reset,
    NotSet,
}

impl<T> Default for Setting<T> {
    fn default() -> Self {
        Self::NotSet
    }
}

impl<T> Setting<T> {
    pub fn set(self) -> Option<T> {
        match self {
            Self::Set(value) => Some(value),
            _ => None,
        }
    }

    pub const fn as_ref(&self) -> Setting<&T> {
        match *self {
            Self::Set(ref value) => Setting::Set(value),
            Self::Reset => Setting::Reset,
            Self::NotSet => Setting::NotSet,
        }
    }

    pub const fn is_not_set(&self) -> bool {
        matches!(self, Self::NotSet)
    }

    /// If `Self` is `Reset`, then map self to `Set` with the provided `val`.
    pub fn or_reset(self, val: T) -> Self {
        match self {
            Self::Reset => Self::Set(val),
            otherwise => otherwise,
        }
    }
}

impl<T: Serialize> Serialize for Setting<T> {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Set(value) => Some(value),
            // Usually not_set isn't serialized by setting skip_serializing_if field attribute
            Self::NotSet | Self::Reset => None,
        }
        .serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Setting<T> {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(|x| match x {
            Some(x) => Self::Set(x),
            None => Self::Reset, // Reset is forced by sending null value
        })
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq, Copy)]
#[serde(rename_all = "camelCase")]
pub struct PaginationSetting {
    pub max_total_hits: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MinWordSizeForTypos {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_typo: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two_typos: Option<u8>,
}

impl Default for MinWordSizeForTypos {
    fn default() -> Self {
        MinWordSizeForTypos {
            one_typo: Some(5),
            two_typos: Some(9),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]

pub struct TypoToleranceSettings {
    #[serde(skip_serializing_if = "Setting::is_not_set")]
    pub enabled: Setting<bool>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    pub disable_on_attributes: Vec<String>,
    //#[serde(skip_serializing_if = "Option::is_none")]
    pub disable_on_words: Vec<String>,
    pub min_word_size_for_typos: MinWordSizeForTypos,
}

impl Default for TypoToleranceSettings {
    fn default() -> Self {
        TypoToleranceSettings {
            enabled: Setting::Set(true),
            disable_on_attributes: vec![],
            disable_on_words: vec![],
            min_word_size_for_typos: MinWordSizeForTypos::default(),
        }
    }
}

impl TypoToleranceSettings {
    pub fn new() -> Self {
        TypoToleranceSettings {
            enabled: Setting::Set(true),
            disable_on_attributes: vec![],
            disable_on_words: vec![],
            min_word_size_for_typos: MinWordSizeForTypos::default(),
        }
    }

    pub fn with_enabled(self, enabled: bool) -> TypoToleranceSettings {
        TypoToleranceSettings {
            enabled: Setting::Set(enabled),
            ..self
        }
    }

    pub fn with_disable_on_attributes(
        self,
        disable_on_attributes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> TypoToleranceSettings {
        TypoToleranceSettings {
            disable_on_attributes: 
                disable_on_attributes
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            
            ..self
        }
    }
    pub fn with_disable_on_words(
        self,
        disable_on_words: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> TypoToleranceSettings {
        TypoToleranceSettings {
            disable_on_words: 
                disable_on_words
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            
            ..self
        }
    }

    pub fn with_min_word_size_for_typos(
        self,
        min_word_size_for_typos: MinWordSizeForTypos,
    ) -> TypoToleranceSettings {
        TypoToleranceSettings {
            min_word_size_for_typos: min_word_size_for_typos,
            ..self
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FacetingSettings {
    #[serde()]
    pub max_values_per_facet: usize,
}

/// Struct reprensenting a set of settings.
/// You can build this struct using the builder syntax.
///
/// # Example
///
/// ```
/// # use meilisearch_sdk::settings::Settings;
/// let settings = Settings::new()
///     .with_stop_words(["a", "the", "of"]);
///
/// // OR
///
/// let stop_words: Vec<String> = vec!["a".to_string(), "the".to_string(), "of".to_string()];
/// let mut settings = Settings::new();
/// settings.stop_words = Some(stop_words);
///
/// // OR
///
/// let stop_words: Vec<String> = vec!["a".to_string(), "the".to_string(), "of".to_string()];
/// let settings = Settings {
///     stop_words: Some(stop_words),
///     ..Settings::new()
/// };
/// ```
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// List of associated words treated similarly
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synonyms: Option<HashMap<String, Vec<String>>>,
    /// List of words ignored by Meilisearch when present in search queries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_words: Option<Vec<String>>,
    /// List of [ranking rules](https://docs.meilisearch.com/learn/core_concepts/relevancy.html#order-of-the-rules) sorted by order of importance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking_rules: Option<Vec<String>>,
    /// Attributes to use for [filtering and faceted search](https://docs.meilisearch.com/reference/features/filtering_and_faceted_search.html)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filterable_attributes: Option<Vec<String>>,
    /// Attributes to sort
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortable_attributes: Option<Vec<String>>,
    /// Search returns documents with distinct (different) values of the given field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distinct_attribute: Option<String>,
    /// Fields in which to search for matching query words sorted by order of importance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searchable_attributes: Option<Vec<String>>,
    /// Fields displayed in the returned documents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayed_attributes: Option<Vec<String>>,
    /// Pagination settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationSetting>,
    /// Faceting settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faceting: Option<FacetingSettings>,
    /// TypoTolerance settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typo_tolerance: Option<TypoToleranceSettings>,
}

#[allow(missing_docs)]
impl Settings {
    /// Create undefined settings
    pub fn new() -> Settings {
        Settings {
            synonyms: None,
            stop_words: None,
            ranking_rules: None,
            filterable_attributes: None,
            sortable_attributes: None,
            distinct_attribute: None,
            searchable_attributes: None,
            displayed_attributes: None,
            pagination: None,
            faceting: None,
            typo_tolerance: None,
        }
    }
    pub fn with_synonyms<S, U, V>(self, synonyms: HashMap<S, U>) -> Settings
    where
        S: AsRef<str>,
        V: AsRef<str>,
        U: IntoIterator<Item = V>,
    {
        Settings {
            synonyms: Some(
                synonyms
                    .into_iter()
                    .map(|(key, value)| {
                        (
                            key.as_ref().to_string(),
                            value.into_iter().map(|v| v.as_ref().to_string()).collect(),
                        )
                    })
                    .collect(),
            ),
            ..self
        }
    }

    pub fn with_stop_words(
        self,
        stop_words: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Settings {
        Settings {
            stop_words: Some(
                stop_words
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            ),
            ..self
        }
    }

    pub fn with_pagination(self, pagination_settings: PaginationSetting) -> Settings {
        Settings {
            pagination: Some(pagination_settings),
            ..self
        }
    }

    pub fn with_typo_tolerance(self, typo_tolerance_settings: TypoToleranceSettings) -> Settings {
        Settings {
            typo_tolerance: Some(typo_tolerance_settings),
            ..self
        }
    }

    pub fn with_ranking_rules(
        self,
        ranking_rules: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Settings {
        Settings {
            ranking_rules: Some(
                ranking_rules
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            ),
            ..self
        }
    }

    pub fn with_filterable_attributes(
        self,
        filterable_attributes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Settings {
        Settings {
            filterable_attributes: Some(
                filterable_attributes
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            ),
            ..self
        }
    }

    pub fn with_sortable_attributes(
        self,
        sortable_attributes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Settings {
        Settings {
            sortable_attributes: Some(
                sortable_attributes
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            ),
            ..self
        }
    }

    pub fn with_distinct_attribute(self, distinct_attribute: impl AsRef<str>) -> Settings {
        Settings {
            distinct_attribute: Some(distinct_attribute.as_ref().to_string()),
            ..self
        }
    }

    pub fn with_searchable_attributes(
        self,
        searchable_attributes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Settings {
        Settings {
            searchable_attributes: Some(
                searchable_attributes
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            ),
            ..self
        }
    }

    pub fn with_displayed_attributes(
        self,
        displayed_attributes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Settings {
        Settings {
            displayed_attributes: Some(
                displayed_attributes
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            ),
            ..self
        }
    }

    pub fn with_faceting(self, faceting: &FacetingSettings) -> Settings {
        Settings {
            faceting: Some(faceting.clone()),
            ..self
        }
    }
}

impl Index {
    /// Get [Settings] of the [Index].
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("get_settings", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let index = client.index("get_settings");
    /// let settings = index.get_settings().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn get_settings(&self) -> Result<Settings, Error> {
        request::<(), Settings>(
            &format!("{}/indexes/{}/settings", self.client.host, self.uid),
            &self.client.api_key,
            Method::Get(()),
            200,
        )
        .await
    }

    /// Get [synonyms](https://docs.meilisearch.com/reference/features/synonyms.html) of the [Index].
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("get_synonyms", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let index = client.index("get_synonyms");
    /// let synonyms = index.get_synonyms().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn get_synonyms(&self) -> Result<HashMap<String, Vec<String>>, Error> {
        request::<(), HashMap<String, Vec<String>>>(
            &format!(
                "{}/indexes/{}/settings/synonyms",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Get(()),
            200,
        )
        .await
    }

    /// Get [pagination](https://docs.meilisearch.com/learn/configuration/settings.html#pagination) of the [Index].
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("get_pagination", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let index = client.index("get_pagination");
    /// let pagination = index.get_pagination().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn get_pagination(&self) -> Result<PaginationSetting, Error> {
        request::<(), PaginationSetting>(
            &format!(
                "{}/indexes/{}/settings/pagination",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Get(()),
            200,
        )
        .await
    }

    /// Get [stop-words](https://docs.meilisearch.com/reference/features/stop_words.html) of the [Index].
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("get_stop_words", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let index = client.index("get_stop_words");
    /// let stop_words = index.get_stop_words().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn get_stop_words(&self) -> Result<Vec<String>, Error> {
        request::<(), Vec<String>>(
            &format!(
                "{}/indexes/{}/settings/stop-words",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Get(()),
            200,
        )
        .await
    }

    /// Get [ranking rules](https://docs.meilisearch.com/learn/core_concepts/relevancy.html#ranking-rules) of the [Index].
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("get_ranking_rules", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let index = client.index("get_ranking_rules");
    /// let ranking_rules = index.get_ranking_rules().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn get_ranking_rules(&self) -> Result<Vec<String>, Error> {
        request::<(), Vec<String>>(
            &format!(
                "{}/indexes/{}/settings/ranking-rules",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Get(()),
            200,
        )
        .await
    }

    /// Get [filterable attributes](https://docs.meilisearch.com/reference/features/filtering_and_faceted_search.html) of the [Index].
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("get_filterable_attributes", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let index = client.index("get_filterable_attributes");
    /// let filterable_attributes = index.get_filterable_attributes().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn get_filterable_attributes(&self) -> Result<Vec<String>, Error> {
        request::<(), Vec<String>>(
            &format!(
                "{}/indexes/{}/settings/filterable-attributes",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Get(()),
            200,
        )
        .await
    }

    /// Get [sortable attributes](https://docs.meilisearch.com/reference/features/sorting.html) of the [Index].
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("get_sortable_attributes", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let index = client.index("get_sortable_attributes");
    /// let sortable_attributes = index.get_sortable_attributes().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn get_sortable_attributes(&self) -> Result<Vec<String>, Error> {
        request::<(), Vec<String>>(
            &format!(
                "{}/indexes/{}/settings/sortable-attributes",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Get(()),
            200,
        )
        .await
    }

    /// Get the [distinct attribute](https://docs.meilisearch.com/reference/features/settings.html#distinct-attribute) of the [Index].
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("get_distinct_attribute", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let index = client.index("get_distinct_attribute");
    /// let distinct_attribute = index.get_distinct_attribute().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn get_distinct_attribute(&self) -> Result<Option<String>, Error> {
        request::<(), Option<String>>(
            &format!(
                "{}/indexes/{}/settings/distinct-attribute",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Get(()),
            200,
        )
        .await
    }

    /// Get [searchable attributes](https://docs.meilisearch.com/reference/features/field_properties.html#searchable-fields) of the [Index].
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("get_searchable_attributes", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let index = client.index("get_searchable_attributes");
    /// let searchable_attributes = index.get_searchable_attributes().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn get_searchable_attributes(&self) -> Result<Vec<String>, Error> {
        request::<(), Vec<String>>(
            &format!(
                "{}/indexes/{}/settings/searchable-attributes",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Get(()),
            200,
        )
        .await
    }

    /// Get [displayed attributes](https://docs.meilisearch.com/reference/features/settings.html#displayed-attributes) of the [Index].
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("get_displayed_attributes", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let index = client.index("get_displayed_attributes");
    /// let displayed_attributes = index.get_displayed_attributes().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn get_displayed_attributes(&self) -> Result<Vec<String>, Error> {
        request::<(), Vec<String>>(
            &format!(
                "{}/indexes/{}/settings/displayed-attributes",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Get(()),
            200,
        )
        .await
    }

    /// Get [faceting](https://docs.meilisearch.com/reference/api/settings.html#faceting) settings of the [Index].
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("get_faceting", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let index = client.index("get_faceting");
    /// let faceting = index.get_faceting().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn get_faceting(&self) -> Result<FacetingSettings, Error> {
        request::<(), FacetingSettings>(
            &format!(
                "{}/indexes/{}/settings/faceting",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Get(()),
            200,
        )
        .await
    }

    /// Get [typo tolerance](https://docs.meilisearch.com/learn/configuration/typo_tolerance.html#typo-tolerance) of the [Index].
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*};
    /// #
    /// # let MEILISEARCH_HOST = option_env!("MEILISEARCH_HOST").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_HOST, MEILISEARCH_API_KEY);
    /// # client.create_index("get_typo_tolerance", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let index = client.index("get_typo_tolerance");
    /// let typotolerance = index.get_typo_tolerance().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn get_typo_tolerance(&self) -> Result<TypoToleranceSettings, Error> {
        request::<(), TypoToleranceSettings>(
            &format!(
                "{}/indexes/{}/settings/typo-tolerance",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Get(()),
            200,
        )
        .await
    }

    /// Update [settings](../settings/struct.Settings.html) of the [Index].
    /// Updates in the settings are partial. This means that any parameters corresponding to a `None` value will be left unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::{Settings, PaginationSetting}};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("set_settings", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("set_settings");
    ///
    /// let stop_words = vec![String::from("a"), String::from("the"), String::from("of")];
    /// let settings = Settings::new()
    ///     .with_stop_words(stop_words.clone())
    ///     .with_pagination(PaginationSetting {max_total_hits: 100}
    /// );
    ///
    /// let task = index.set_settings(&settings).await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn set_settings(&self, settings: &Settings) -> Result<TaskInfo, Error> {
        request::<&Settings, TaskInfo>(
            &format!("{}/indexes/{}/settings", self.client.host, self.uid),
            &self.client.api_key,
            Method::Patch(settings),
            202,
        )
        .await
    }

    /// Update [synonyms](https://docs.meilisearch.com/reference/features/synonyms.html) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("set_synonyms", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("set_synonyms");
    ///
    /// let mut synonyms = std::collections::HashMap::new();
    /// synonyms.insert(String::from("wolverine"), vec![String::from("xmen"), String::from("logan")]);
    /// synonyms.insert(String::from("logan"), vec![String::from("xmen"), String::from("wolverine")]);
    /// synonyms.insert(String::from("wow"), vec![String::from("world of warcraft")]);
    ///
    /// let task = index.set_synonyms(&synonyms).await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn set_synonyms(
        &self,
        synonyms: &HashMap<String, Vec<String>>,
    ) -> Result<TaskInfo, Error> {
        request::<&HashMap<String, Vec<String>>, TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/synonyms",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Put(synonyms),
            202,
        )
        .await
    }

    /// Update [pagination](https://docs.meilisearch.com/learn/configuration/settings.html#pagination) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::{Settings, PaginationSetting}};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("set_pagination", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("set_pagination");
    /// let pagination = PaginationSetting {max_total_hits:100};
    /// let task = index.set_pagination(pagination).await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn set_pagination(&self, pagination: PaginationSetting) -> Result<TaskInfo, Error> {
        request::<&PaginationSetting, TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/pagination",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Patch(&pagination),
            202,
        )
        .await
    }

    /// Update [stop-words](https://docs.meilisearch.com/reference/features/stop_words.html) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("set_stop_words", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("set_stop_words");
    ///
    /// let stop_words = ["the", "of", "to"];
    /// let task = index.set_stop_words(&stop_words).await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn set_stop_words(
        &self,
        stop_words: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<TaskInfo, Error> {
        request::<Vec<String>, TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/stop-words",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Put(
                stop_words
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            ),
            202,
        )
        .await
    }

    /// Update [ranking rules](https://docs.meilisearch.com/learn/core_concepts/relevancy.html#ranking-rules) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("set_ranking_rules", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("set_ranking_rules");
    ///
    /// let ranking_rules = [
    ///     "words",
    ///     "typo",
    ///     "proximity",
    ///     "attribute",
    ///     "sort",
    ///     "exactness",
    ///     "release_date:asc",
    ///     "rank:desc",
    /// ];
    /// let task = index.set_ranking_rules(ranking_rules).await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn set_ranking_rules(
        &self,
        ranking_rules: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<TaskInfo, Error> {
        request::<Vec<String>, TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/ranking-rules",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Put(
                ranking_rules
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            ),
            202,
        )
        .await
    }

    /// Update [filterable attributes](https://docs.meilisearch.com/reference/features/filtering_and_faceted_search.html) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("set_filterable_attributes", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("set_filterable_attributes");
    ///
    /// let filterable_attributes = ["genre", "director"];
    /// let task = index.set_filterable_attributes(&filterable_attributes).await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn set_filterable_attributes(
        &self,
        filterable_attributes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<TaskInfo, Error> {
        request::<Vec<String>, TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/filterable-attributes",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Put(
                filterable_attributes
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            ),
            202,
        )
        .await
    }

    /// Update [sortable attributes](https://docs.meilisearch.com/reference/features/sorting.html) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("set_sortable_attributes", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("set_sortable_attributes");
    ///
    /// let sortable_attributes = ["genre", "director"];
    /// let task = index.set_sortable_attributes(&sortable_attributes).await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn set_sortable_attributes(
        &self,
        sortable_attributes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<TaskInfo, Error> {
        request::<Vec<String>, TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/sortable-attributes",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Put(
                sortable_attributes
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            ),
            202,
        )
        .await
    }

    /// Update the [distinct attribute](https://docs.meilisearch.com/reference/features/settings.html#distinct-attribute) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("set_distinct_attribute", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("set_distinct_attribute");
    ///
    /// let task = index.set_distinct_attribute("movie_id").await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn set_distinct_attribute(
        &self,
        distinct_attribute: impl AsRef<str>,
    ) -> Result<TaskInfo, Error> {
        request::<String, TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/distinct-attribute",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Put(distinct_attribute.as_ref().to_string()),
            202,
        )
        .await
    }

    /// Update [searchable attributes](https://docs.meilisearch.com/reference/features/field_properties.html#searchable-fields) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("set_searchable_attributes", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("set_searchable_attributes");
    ///
    /// let task = index.set_searchable_attributes(["title", "description", "uid"]).await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn set_searchable_attributes(
        &self,
        searchable_attributes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<TaskInfo, Error> {
        request::<Vec<String>, TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/searchable-attributes",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Put(
                searchable_attributes
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            ),
            202,
        )
        .await
    }

    /// Update [displayed attributes](https://docs.meilisearch.com/reference/features/settings.html#displayed-attributes) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("set_displayed_attributes", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("set_displayed_attributes");
    ///
    /// let task = index.set_displayed_attributes(["title", "description", "release_date", "rank", "poster"]).await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn set_displayed_attributes(
        &self,
        displayed_attributes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<TaskInfo, Error> {
        request::<Vec<String>, TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/displayed-attributes",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Put(
                displayed_attributes
                    .into_iter()
                    .map(|v| v.as_ref().to_string())
                    .collect(),
            ),
            202,
        )
        .await
    }

    /// Update [faceting](https://docs.meilisearch.com/reference/api/settings.html#faceting) settings of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings, settings::FacetingSettings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("set_faceting", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("set_faceting");
    ///
    /// let mut faceting = FacetingSettings {
    ///     max_values_per_facet: 12,
    /// };
    ///
    /// let task = index.set_faceting(&faceting).await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn set_faceting(&self, faceting: &FacetingSettings) -> Result<TaskInfo, Error> {
        request::<&FacetingSettings, TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/faceting",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Patch(faceting),
            202,
        )
        .await
    }

    /// Update [typo tolerance](https://docs.meilisearch.com/learn/configuration/typo_tolerance.html#typo-tolerance) settings of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings, settings::{TypoToleranceSettings, MinWordSizeForTypos}};
    /// #
    /// # let MEILISEARCH_HOST = option_env!("MEILISEARCH_HOST").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_HOST, MEILISEARCH_API_KEY);
    /// # client.create_index("set_typo_tolerance", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("set_typo_tolerance");
    ///
    /// let mut typo_tolerance = TypoToleranceSettings::new().with_enabled(false);
    ///
    /// let task = index.set_typo_tolerance(&typo_tolerance).await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn set_typo_tolerance(
        &self,
        typo_tolerance: &TypoToleranceSettings,
    ) -> Result<TaskInfo, Error> {
        request::<&TypoToleranceSettings, TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/typo-tolerance",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Patch(typo_tolerance),
            202,
        )
        .await
    }

    /// Reset [Settings] of the [Index].
    /// All settings will be reset to their [default value](https://docs.meilisearch.com/reference/api/settings.html#reset-settings).
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("reset_settings", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("reset_settings");
    ///
    /// let task = index.reset_settings().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn reset_settings(&self) -> Result<TaskInfo, Error> {
        request::<(), TaskInfo>(
            &format!("{}/indexes/{}/settings", self.client.host, self.uid),
            &self.client.api_key,
            Method::Delete,
            202,
        )
        .await
    }

    /// Reset [synonyms](https://docs.meilisearch.com/reference/features/synonyms.html) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("reset_synonyms", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("reset_synonyms");
    ///
    /// let task = index.reset_synonyms().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn reset_synonyms(&self) -> Result<TaskInfo, Error> {
        request::<(), TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/synonyms",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Delete,
            202,
        )
        .await
    }

    /// Reset [pagination](https://docs.meilisearch.com/learn/configuration/settings.html#pagination) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("reset_pagination", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("reset_pagination");
    ///
    /// let task = index.reset_pagination().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn reset_pagination(&self) -> Result<TaskInfo, Error> {
        request::<(), TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/pagination",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Delete,
            202,
        )
        .await
    }
    /// Reset [stop-words](https://docs.meilisearch.com/reference/features/stop_words.html) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("reset_stop_words", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("reset_stop_words");
    ///
    /// let task = index.reset_stop_words().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn reset_stop_words(&self) -> Result<TaskInfo, Error> {
        request::<(), TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/stop-words",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Delete,
            202,
        )
        .await
    }

    /// Reset [ranking rules](https://docs.meilisearch.com/learn/core_concepts/relevancy.html#ranking-rules) of the [Index] to default value.
    /// Default value: `["words", "typo", "proximity", "attribute", "sort", "exactness"]`.
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("reset_ranking_rules", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("reset_ranking_rules");
    ///
    /// let task = index.reset_ranking_rules().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn reset_ranking_rules(&self) -> Result<TaskInfo, Error> {
        request::<(), TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/ranking-rules",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Delete,
            202,
        )
        .await
    }

    /// Reset [filterable attributes](https://docs.meilisearch.com/reference/features/filtering_and_faceted_search.html) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("reset_filterable_attributes", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("reset_filterable_attributes");
    ///
    /// let task = index.reset_filterable_attributes().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn reset_filterable_attributes(&self) -> Result<TaskInfo, Error> {
        request::<(), TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/filterable-attributes",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Delete,
            202,
        )
        .await
    }

    /// Reset [sortable attributes](https://docs.meilisearch.com/reference/features/sorting.html) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("reset_sortable_attributes", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("reset_sortable_attributes");
    ///
    /// let task = index.reset_sortable_attributes().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn reset_sortable_attributes(&self) -> Result<TaskInfo, Error> {
        request::<(), TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/sortable-attributes",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Delete,
            202,
        )
        .await
    }

    /// Reset the [distinct attribute](https://docs.meilisearch.com/reference/features/settings.html#distinct-attribute) of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("reset_distinct_attribute", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("reset_distinct_attribute");
    ///
    /// let task = index.reset_distinct_attribute().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn reset_distinct_attribute(&self) -> Result<TaskInfo, Error> {
        request::<(), TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/distinct-attribute",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Delete,
            202,
        )
        .await
    }

    /// Reset [searchable attributes](https://docs.meilisearch.com/reference/features/field_properties.html#searchable-fields) of the [Index] (enable all attributes).
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("reset_searchable_attributes", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("reset_searchable_attributes");
    ///
    /// let task = index.reset_searchable_attributes().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn reset_searchable_attributes(&self) -> Result<TaskInfo, Error> {
        request::<(), TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/searchable-attributes",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Delete,
            202,
        )
        .await
    }

    /// Reset [displayed attributes](https://docs.meilisearch.com/reference/features/settings.html#displayed-attributes) of the [Index] (enable all attributes).
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("reset_displayed_attributes", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("reset_displayed_attributes");
    ///
    /// let task = index.reset_displayed_attributes().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn reset_displayed_attributes(&self) -> Result<TaskInfo, Error> {
        request::<(), TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/displayed-attributes",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Delete,
            202,
        )
        .await
    }

    /// Reset [faceting](https://docs.meilisearch.com/reference/api/settings.html#faceting) settings of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_URL = option_env!("MEILISEARCH_URL").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_URL, MEILISEARCH_API_KEY);
    /// # client.create_index("reset_faceting", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("reset_faceting");
    ///
    /// let task = index.reset_faceting().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn reset_faceting(&self) -> Result<TaskInfo, Error> {
        request::<(), TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/faceting",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Delete,
            202,
        )
        .await
    }

    /// Reset [typo tolerance](https://docs.meilisearch.com/learn/configuration/typo_tolerance.html#typo-tolerance) settings of the [Index].
    ///
    /// # Example
    ///
    /// ```
    /// # use meilisearch_sdk::{client::*, indexes::*, settings::Settings};
    /// #
    /// # let MEILISEARCH_HOST = option_env!("MEILISEARCH_HOST").unwrap_or("http://localhost:7700");
    /// # let MEILISEARCH_API_KEY = option_env!("MEILISEARCH_API_KEY").unwrap_or("masterKey");
    /// #
    /// # futures::executor::block_on(async move {
    /// let client = Client::new(MEILISEARCH_HOST, MEILISEARCH_API_KEY);
    /// # client.create_index("reset_typo_tolerance", None).await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// let mut index = client.index("reset_typo_tolerance");
    ///
    /// let task = index.reset_typo_tolerance().await.unwrap();
    /// # index.delete().await.unwrap().wait_for_completion(&client, None, None).await.unwrap();
    /// # });
    /// ```
    pub async fn reset_typo_tolerance(&self) -> Result<TaskInfo, Error> {
        request::<(), TaskInfo>(
            &format!(
                "{}/indexes/{}/settings/typo-tolerance",
                self.client.host, self.uid
            ),
            &self.client.api_key,
            Method::Delete,
            202,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::client::*;
    use meilisearch_test_macro::meilisearch_test;

    #[meilisearch_test]
    async fn test_set_faceting_settings(client: Client, index: Index) {
        let faceting = FacetingSettings {
            max_values_per_facet: 5,
        };
        let settings = Settings::new().with_faceting(&faceting);

        let task_info = index.set_settings(&settings).await.unwrap();
        client.wait_for_task(task_info, None, None).await.unwrap();

        let res = index.get_faceting().await.unwrap();

        assert_eq!(faceting, res);
    }

    #[meilisearch_test]
    async fn test_get_faceting(index: Index) {
        let faceting = FacetingSettings {
            max_values_per_facet: 100,
        };

        let res = index.get_faceting().await.unwrap();

        assert_eq!(faceting, res);
    }

    #[meilisearch_test]
    async fn test_set_faceting(client: Client, index: Index) {
        let faceting = FacetingSettings {
            max_values_per_facet: 5,
        };
        let task_info = index.set_faceting(&faceting).await.unwrap();
        client.wait_for_task(task_info, None, None).await.unwrap();

        let res = index.get_faceting().await.unwrap();

        assert_eq!(faceting, res);
    }

    #[meilisearch_test]
    async fn test_reset_faceting(client: Client, index: Index) {
        let task_info = index.reset_faceting().await.unwrap();
        client.wait_for_task(task_info, None, None).await.unwrap();
        let faceting = FacetingSettings {
            max_values_per_facet: 100,
        };

        let res = index.get_faceting().await.unwrap();

        assert_eq!(faceting, res);
    }

    #[meilisearch_test]
    async fn test_get_pagination(index: Index) {
        let pagination = PaginationSetting {
            max_total_hits: 1000,
        };

        let res = index.get_pagination().await.unwrap();

        assert_eq!(pagination, res);
    }

    #[meilisearch_test]
    async fn test_set_pagination(index: Index) {
        let pagination = PaginationSetting { max_total_hits: 11 };
        let task = index.set_pagination(pagination).await.unwrap();
        index.wait_for_task(task, None, None).await.unwrap();

        let res = index.get_pagination().await.unwrap();

        assert_eq!(pagination, res);
    }

    #[meilisearch_test]
    async fn test_reset_pagination(index: Index) {
        let pagination = PaginationSetting { max_total_hits: 10 };
        let default = PaginationSetting {
            max_total_hits: 1000,
        };

        let task = index.set_pagination(pagination).await.unwrap();
        index.wait_for_task(task, None, None).await.unwrap();

        let reset_task = index.reset_pagination().await.unwrap();
        index.wait_for_task(reset_task, None, None).await.unwrap();

        let res = index.get_pagination().await.unwrap();

        assert_eq!(default, res);
    }

    #[meilisearch_test]
    async fn test_get_typo_tolerance(index: Index) {
        let typo_tolerance = TypoToleranceSettings::default();

        let res = index.get_typo_tolerance().await.unwrap();

        assert_eq!(typo_tolerance, res);
    }

    #[meilisearch_test]
    async fn test_set_typo_tolerance(client: Client, index: Index) {
        let typo_tolerance = TypoToleranceSettings::new().
            with_disable_on_attributes(vec!["title"]).with_enabled(false);

        let task_info = index.set_typo_tolerance(&typo_tolerance).await.unwrap();
        client.wait_for_task(task_info, None, None).await.unwrap();

        let res = index.get_typo_tolerance().await.unwrap();

        assert_eq!(typo_tolerance, res);
    }

    #[meilisearch_test]
    async fn test_reset_typo_tolerance(index: Index) {
        let typo_tolerance = TypoToleranceSettings::new().with_enabled(false);

        let task = index.set_typo_tolerance(&typo_tolerance).await.unwrap();
        index.wait_for_task(task, None, None).await.unwrap();

        let reset_task = index.reset_typo_tolerance().await.unwrap();
        index.wait_for_task(reset_task, None, None).await.unwrap();

        let res = index.get_typo_tolerance().await.unwrap();

        assert_eq!(TypoToleranceSettings::new(), res);
    }
}
