<ui>
  <!-- 底部限购栏：今日限购次数文案 + 购买进度条（ProgressBar value 由业务数据驱动） -->
  <style>
	.footer-text { color: #6b5a3e; font_size: 15; }
	.purchase-bar {
	  background: #e8b93e;
	  border_radius: 8;
	}
	.purchase-track {
	  background: #e4d9bc;
	  border_color: #c9b98c;
	  border_width: 1;
	  border_radius: 8;
	}
  </style>
  <Panel name="StoreFooter" custom_minimum_size="0,40">
	<HBoxContainer anchor="full" margin="4 4 4 4">
	  <Label class="footer-text" text="📜 今日限购  2/5" valign="center" />
	  <Control size_flags_horizontal="expand_fill" />
	  <ProgressBar name="PurchaseBar" value="40" percent_visible="false"
	           custom_minimum_size="420,18" size_flags_vertical="shrink_center">
		<style background="purchase-bar" track="purchase-track" />
	  </ProgressBar>
	</HBoxContainer>
  </Panel>
</ui>
