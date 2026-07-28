#ifndef _H_IRISCONTINUESTATEMENT_
#define _H_IRISCONTINUESTATEMENT_
#include "IrisStatement.h"

class IrisIdentifier;
class IrisContinueStatement :
	public IrisStatement
{
public:
	IrisIdentifier* m_pLabel = nullptr;

public:

	virtual bool Generate();

	IrisContinueStatement(IrisIdentifier* pLabel);
	virtual ~IrisContinueStatement();

	virtual bool Validate() override;
};

#endif;