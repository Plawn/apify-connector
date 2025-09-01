use serde::{Deserialize, Serialize};

use crate::client::DatasetId;

#[derive(Debug, Deserialize)]
pub struct RunId(pub String);

#[derive(Debug, Deserialize)]
pub struct Data {
    pub actId: String,
    pub buildId: String,
    pub buildNumber: String,
    pub containerUrl: String,
    pub defaultDatasetId: DatasetId,
    pub defaultKeyValueStoreId: String,
    pub defaultRequestQueueId: String,
    pub finishedAt: Option<String>, // Utilisation de Option pour gérer `null`
    pub generalAccess: String,
    pub id: RunId,
    pub meta: Meta,
    pub options: Options,
    pub platformUsageBillingModel: String,
    pub pricingInfo: PricingInfo,
    pub startedAt: String,
    pub stats: Stats,
    pub status: String,
    pub userId: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    pub origin: String,
    pub userAgent: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Options {
    build: String,
    pub diskMbytes: u32,
    pub maxItems: u32,
    pub memoryMbytes: u32,
    pub timeoutSecs: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PricingInfo {
    pub apifyMarginPercentage: f64,
    pub createdAt: String,
    pub pricePerUnitUsd: f64,
    pub pricingModel: String,
    pub startedAt: String,
    pub unitName: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Stats {
    pub computeUnits: u32,
    pub inputBodyLen: u32,
    pub migrationCount: u32,
    pub rebootCount: u32,
    pub restartCount: u32,
    pub resurrectCount: u32,
}

#[derive(Debug, Deserialize)]
pub struct Root {
    pub data: Data,
}




/*
ex item
  {
    "id": "997053569",
    "url": "https://www.tripadvisor.com/ShowUserReviews-g182183-d2311154-r997053569-Casino_De_Mont_tremblant-Mont_Tremblant_Quebec.html",
    "title": "Passez votre chemin.",
    "lang": "fr",
    "locationId": "2311154",
    "publishedDate": "2025-03-07",
    "publishedPlatform": "OTHER",
    "rating": 2,
    "helpfulVotes": 0,
    "text": "Déception totale. Aucune ambiance. Plus de sécurité que de client. Problématique vécue avec une machine et après 4 appels non répondus au personnel j’ai du aller demander au bar… dans le doute(camera) le casino na pas accommode client malgré somme mineure! stationnement extérieur gratuit. Aucun breuvage gratuit.",
    "roomTip": null,
    "travelDate": "2025-03",
    "tripType": "COUPLES",
    "user": {
      "userId": "0D5C39776DAF248F1E499A293EA8D483",
      "name": "Tania D",
      "contributions": {
        "totalContributions": 155,
        "helpfulVotes": 102
      },
      "username": "carria_us",
      "userLocation": {
        "shortName": "Quebec City",
        "name": "Quebec City, Canada",
        "id": "155033"
      },
      "avatar": {
        "id": "19820645",
        "width": 205,
        "height": 205,
        "image": "https://dynamic-media-cdn.tripadvisor.com/media/photo-f/01/2e/70/65/avatar043.jpg"
      },
      "link": "www.tripadvisor.com/Profile/carria_us"
    },
    "ownerResponse": null,
    "subratings": [],
    "photos": [],
    "placeInfo": {
      "id": "2311154",
      "name": "Casino De Mont-tremblant",
      "rating": 3.5,
      "numberOfReviews": 592,
      "locationString": "Mont Tremblant, Quebec",
      "latitude": 46.200977,
      "longitude": -74.56839,
      "webUrl": "https://www.tripadvisor.com/Attraction_Review-g182183-d2311154-Reviews-Casino_De_Mont_tremblant-Mont_Tremblant_Quebec.html",
      "website": "https://casinos.lotoquebec.com/fr/monttremblant/restaurant/altitude",
      "address": "300 Chemin des Pleiades, Mont Tremblant, Quebec J8E 0A7 Canada",
      "addressObj": {
        "street1": "300 Chemin des Pleiades",
        "street2": "",
        "city": "Mont Tremblant",
        "state": null,
        "country": "Canada",
        "postalcode": "J8E 0A7"
      },
      "ratingHistogram": {
        "count1": 27,
        "count2": 61,
        "count3": 218,
        "count4": 176,
        "count5": 110
      }
    }
  },
*/