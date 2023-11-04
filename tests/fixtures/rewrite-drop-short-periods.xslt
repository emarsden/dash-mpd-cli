<?xml version="1.0" encoding="utf-8"?>
<xsl:stylesheet version="1.0"
                xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
                xmlns:mpd="urn:mpeg:dash:schema:mpd:2011">
  <xsl:output method="xml" indent="yes"/>

  <!-- Default action (unless a template below matches): copy -->
  <xsl:template match="@*|node()">
    <xsl:copy>
      <xsl:apply-templates select="@*|node()"/>
    </xsl:copy>
  </xsl:template>

  <!--
      Drop any Period whose length is less than 6 seconds (probably an ad).

      Unfortunately XSLT 1.0 has no support for date formats; this requires XSLT 3.0 so won't work
      with xsltproc which we currently use to apply stylesheets.
  -->
  <xsl:template match="//node()[local-name()='Period']">
    <xsl:if test="seconds-from-duration(@duration) > 6"> 
      <xsl:copy>
        <xsl:apply-templates select="@*|node()"/>
      </xsl:copy>
    </xsl:if>
  </xsl:template>
</xsl:stylesheet>
