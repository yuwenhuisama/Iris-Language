#ifndef _H_IRISSWITCHSTATEMENT_
#define _H_IRISSWITCHSTATEMENT_

#include "IrisStatement.h"

class IrisExpression;
class IrisSwitchBlock;
class IrisSwitchStatement :
	public IrisStatement
{
protected:
	IrisExpression* m_pCondition = nullptr;
	IrisSwitchBlock* m_pSwitchBlock = nullptr;

	//struct _WhenStructure;

public:

	virtual bool Generate();

	IrisSwitchStatement(IrisExpression* pCondition, IrisSwitchBlock* pSwitchBlock);
	virtual ~IrisSwitchStatement();
	
	virtual bool Validate() override;

};

#endif