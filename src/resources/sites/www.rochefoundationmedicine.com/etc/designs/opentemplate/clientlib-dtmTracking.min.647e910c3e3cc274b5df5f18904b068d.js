"use strict";
var digitalData=digitalData||{};
digitalData.page=digitalData.page||{};
digitalData.page.pageInfo=digitalData.page.pageInfo||{};
digitalData.page.category=digitalData.page.category||{};
digitalData.page.attributes=digitalData.page.attributes||{};
digitalData.page.attributes.onsiteSearch=digitalData.page.attributes.onsiteSearch||{};
digitalData.page.formInfo=digitalData.page.formInfo||{};
digitalData.user=digitalData.user||{};
digitalData.user.segment=digitalData.user.segment||{};
digitalData.user.profile=digitalData.user.profile||{};
digitalData.event=digitalData.event||{};
digitalData.event.video=digitalData.event.video||{};
var addPropertyIfNotEmptyValue=function(a,c,b){if(typeof(b)!=="undefined"&&b!==""){a[c]=b
}};
var removeProperty=function(a,b){delete a[b]
};
var trackGlobalPageInformation=function(a){addPropertyIfNotEmptyValue(digitalData.page.pageInfo,"pageName",a.data("page-name"));
addPropertyIfNotEmptyValue(digitalData.page.pageInfo,"pageType",a.data("page-type"));
addPropertyIfNotEmptyValue(digitalData.page.pageInfo,"destinationURL",a.data("destination-url"));
addPropertyIfNotEmptyValue(digitalData.page.pageInfo,"siteName",a.data("site-name"));
addPropertyIfNotEmptyValue(digitalData.page.pageInfo,"sysEnv",a.data("sys-env"));
addPropertyIfNotEmptyValue(digitalData.page.category,"primaryCategory",a.data("primary-category"));
addPropertyIfNotEmptyValue(digitalData.page.category,"subCategory1",a.data("sub-category1"));
addPropertyIfNotEmptyValue(digitalData.page.category,"subCategory2",a.data("sub-category2"));
addPropertyIfNotEmptyValue(digitalData.page.category,"subCategory3",a.data("sub-category3"));
addPropertyIfNotEmptyValue(digitalData.page.attributes,"effectiveDate",a.data("effective-date"));
addPropertyIfNotEmptyValue(digitalData.page.attributes,"country",a.data("country"));
addPropertyIfNotEmptyValue(digitalData.page.attributes,"language",a.data("language"));
addPropertyIfNotEmptyValue(digitalData.page.attributes,"journalName",a.data("journal-name"));
var b=[];
$("[data-analytics-page-carousel]").each(function(){var d=$(this);
var c=d.data("analytics-page-carousel");
b.push(c)
});
addPropertyIfNotEmptyValue(digitalData.page.attributes,"carousel",b)
};
var trackOnsiteSearchInformation=function(a){if(a.length>0){addPropertyIfNotEmptyValue(digitalData.page.attributes.onsiteSearch,"term",a.data("term"));
addPropertyIfNotEmptyValue(digitalData.page.attributes.onsiteSearch,"results",a.data("results"));
var b=[];
if(a.attr("data-date-from")){b.push({type:"dateFrom",value:a.data("date-from")})
}if(a.attr("data-date-to")){b.push({type:"dateTo",value:a.data("date-to")})
}if(a.attr("data-tag")){b.push({type:"tag",value:a.data("tag")})
}addPropertyIfNotEmptyValue(digitalData.page.attributes.onsiteSearch,"filter",b)
}};
var trackFormInformation=function(c,b){if(c.length>0){var a=c.closest("form");
addPropertyIfNotEmptyValue(digitalData.page.formInfo,"formName",c.attr("id"));
addPropertyIfNotEmptyValue(digitalData.page.formInfo,"step","1");
addPropertyIfNotEmptyValue(digitalData.page.formInfo,"totalSteps","1");
addPropertyIfNotEmptyValue(digitalData.page.formInfo,"isError",a.find("p.form_error").length>0?"true":"false")
}if(b.length>0){addPropertyIfNotEmptyValue(digitalData.page.formInfo,"offerName",b.data("form-offer-name"));
addPropertyIfNotEmptyValue(digitalData.page.formInfo,"isSubmitted",b.data("form-is-submitted"));
addPropertyIfNotEmptyValue(digitalData.page.formInfo,"isConfirmed",b.data("form-is-confirmed"))
}};
var trackUserInformation=function(){addPropertyIfNotEmptyValue(digitalData.user.segment,"loggedIn","false")
};
var trackLinks=function(){$("[data-analytics-page-carousel]").each(function(){var b=$(this);
var a=b.data("analytics-page-carousel");
b.find($("a")).each(function(){b=$(this);
b.attr("data-analytics-link-carousel",a)
})
});
$("[data-content-featured='true']").each(function(){var a=$(this);
a.find($("a")).each(function(){a=$(this);
a.attr("data-analytics-link-carousel","true")
})
});
$("input[type='submit']").each(function(){var a=$(this);
a.attr("data-analytics-link-action","formsubmit")
})
};
var trackYouTubeVideos=function(){var a=document.createElement("script");
a.id="iframe-demo";
a.src="https://www.youtube.com/iframe_api";
var b=document.getElementsByTagName("script")[0];
b.parentNode.insertBefore(a,b)
};
var onYouTubeIframeAPIReady=function(){var a=function(c){var b=c.data;
if(b==YT.PlayerState.PLAYING){if(typeof(digitalData.event.video.paused)==="undefined"){addPropertyIfNotEmptyValue(digitalData.event.video,"opened",{videoName:c.target.getVideoData().title,totalTime:c.target.getDuration(),playerName:"Youtube Player"});
removeProperty(digitalData.event.video,"closed")
}else{addPropertyIfNotEmptyValue(digitalData.event.video,"resumed",{videoName:c.target.getVideoData().title,currentTime:c.target.getCurrentTime()});
removeProperty(digitalData.event.video,"paused")
}}else{if(b==YT.PlayerState.PAUSED){addPropertyIfNotEmptyValue(digitalData.event.video,"paused",{videoName:c.target.getVideoData().title,currentTime:c.target.getCurrentTime()});
removeProperty(digitalData.event.video,"resumed")
}else{if(b==YT.PlayerState.ENDED){addPropertyIfNotEmptyValue(digitalData.event.video,"closed",{videoName:c.target.getVideoData().title,currentTime:c.target.getCurrentTime()});
removeProperty(digitalData.event.video,"opened")
}}}};
$(".youtube-video").each(function(){var c=$(this).attr("id");
var b=new YT.Player(c,{events:{onStateChange:a}})
})
};
var $dtmTracking=$(".dtm-tracking").first();
var $dtmOnsiteSearch=$(".dtm-onsiteSearch").first();
var $dtmForm=$("input[name=':formid']").first();
trackGlobalPageInformation($dtmTracking);
trackOnsiteSearchInformation($dtmOnsiteSearch);
trackFormInformation($dtmForm,$dtmTracking);
trackUserInformation();
trackLinks();
trackYouTubeVideos();